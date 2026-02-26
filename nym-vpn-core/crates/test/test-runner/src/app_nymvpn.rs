// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use chrono::{DateTime, Utc};
use std::path::PathBuf;

use test_rpc::{AppTrace, Error};

pub(crate) const NYMVPND_CLI_BIN: &str = "/opt/testing/nym-vpnd";

/// Get the installed app version string
pub async fn version() -> Result<String, Error> {
    // use the absolute path in case the binary isn't in PATH
    let version = tokio::process::Command::new(NYMVPND_CLI_BIN)
        .arg("--version")
        .output()
        .await
        .map_err(|e| {
            Error::ServiceNotFound(format!(
                "Failed to get version of {}: {}",
                NYMVPND_CLI_BIN, e
            ))
        })?;
    let stdout = String::from_utf8(version.stdout).map_err(|err| Error::Other(err.to_string()))?;

    parse_version_from_output(stdout)
}

/// output from `nym-vpnd --version` looks like this so we need to parse it
// nym-vpnd
// Binary Name:        nym-vpnd
// Build Timestamp:    2025-08-26T11:49:35.310099884Z
// Build Version:      1.14.0
// Commit SHA:         035b20a0a2bf5e35875afe308d93501c981bff69
// Commit Date:        2025-08-26T14:32:45.000000000+03:00
// Commit Branch:      release/2025.13-apricot-banana
// rustc Version:      1.88.0
// rustc Channel:      stable
// cargo Profile:      release
fn parse_version_from_output(stdout: String) -> Result<String, Error> {
    let version_line = stdout
        .lines()
        .find(|line| {
            line.to_lowercase()
                .trim_start()
                .starts_with("build version:")
        })
        .ok_or("`Build Version:` line not found".to_string())
        .map_err(Error::Other)?;
    let version = version_line
        .split_once(':')
        .map(|x| x.1)
        .map(str::trim)
        .ok_or("malformed `Build Version:` line".to_string())
        .map_err(Error::Other)?;

    let version = version.to_string();
    Ok(version)
}

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_version() {
        const EXPECTED_STDOUT: &str = r#"
nym-vpnd
Binary Name:        nym-vpnd
Build Timestamp:    2025-08-26T11:49:35.310099884Z
Build Version:      1.14.0
Commit SHA:         035b20a0a2bf5e35875afe308d93501c981bff69
Commit Date:        2025-08-26T14:32:45.000000000+03:00
Commit Branch:      release/2025.13-apricot-banana
rustc Version:      1.88.0
rustc Channel:      stable
cargo Profile:      release"#;

        let parsed_version = parse_version_from_output(EXPECTED_STDOUT.to_string())
            .expect("Failed to parse version");

        assert_eq!("1.14.0", parsed_version);
    }
}

#[cfg(target_os = "linux")]
pub fn find_traces() -> Result<Vec<AppTrace>, Error> {
    todo!()
}

#[cfg(target_os = "windows")]
pub fn find_traces() -> Result<Vec<AppTrace>, Error> {
    // TODO: Implement Windows trace detection
    Ok(Vec::new())
}

pub fn find_cache_traces() -> Result<PathBuf, Error> {
    unimplemented!()
}

#[cfg(target_os = "macos")]
pub fn find_traces() -> Result<Vec<AppTrace>, Error> {
    use std::path::Path;

    // TODO: Check GUI data
    // TODO: Check temp data

    let caches = find_cache_traces()?;
    let traces = vec![
        Path::new(r"/Applications/Mullvad VPN.app/"),
        Path::new(r"/var/log/mullvad-vpn/"),
        &caches,
        // management interface socket
        Path::new(r"/var/run/mullvad-vpn"),
        // launch daemon
        Path::new(r"/Library/LaunchDaemons/net.mullvad.daemon.plist"),
        Path::new(r"/usr/local/bin/mullvad"),
        Path::new(r"/usr/local/bin/mullvad-problem-report"),
        // completions
        Path::new(r"/usr/local/share/zsh/site-functions/_mullvad"),
        Path::new(r"/opt/homebrew/share/fish/vendor_completions.d/mullvad.fish"),
        Path::new(r"/usr/local/share/fish/vendor_completions.d/mullvad.fish"),
    ];

    Ok(existing_paths(&traces))
}

#[cfg(target_os = "macos")]
/// Find all present app traces on the test runner.
fn existing_paths(paths: &[&std::path::Path]) -> Vec<AppTrace> {
    paths
        .iter()
        .filter(|&path| path.try_exists().is_ok_and(|exists| exists))
        .map(|path| AppTrace::Path(path.to_path_buf()))
        .collect()
}

pub async fn make_device_json_old() -> Result<(), Error> {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    const DEVICE_JSON_PATH: &str = "/etc/mullvad-vpn/device.json";
    #[cfg(target_os = "windows")]
    const DEVICE_JSON_PATH: &str =
        "C:\\Windows\\system32\\config\\systemprofile\\AppData\\Local\\Mullvad VPN\\device.json";
    let device_json = tokio::fs::read_to_string(DEVICE_JSON_PATH)
        .await
        .map_err(|e| Error::FileSystem(e.to_string()))?;

    let mut device_state: serde_json::Value =
        serde_json::from_str(&device_json).map_err(|e| Error::FileSerialization(e.to_string()))?;
    let created_ref: &mut serde_json::Value = device_state
        .get_mut("logged_in")
        .unwrap()
        .get_mut("device")
        .unwrap()
        .get_mut("wg_data")
        .unwrap()
        .get_mut("created")
        .unwrap();
    let created: DateTime<Utc> = serde_json::from_value(created_ref.clone()).unwrap();
    let created = created
        .checked_sub_signed(chrono::Duration::days(365))
        .unwrap();

    *created_ref = serde_json::to_value(created).unwrap();

    let device_json = serde_json::to_string(&device_state)
        .map_err(|e| Error::FileSerialization(e.to_string()))?;
    tokio::fs::write(DEVICE_JSON_PATH, device_json.as_bytes())
        .await
        .map_err(|e| Error::FileSystem(e.to_string()))?;

    Ok(())
}
