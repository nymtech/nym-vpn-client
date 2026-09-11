// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(target_os = "linux")]
use std::collections::HashMap;
use test_rpc::nym_daemon::Verbosity;

/// Drop-in path must match the unit name (`nymvpnd.service`), not a Mullvad leftover.
pub(crate) const NYM_VPN_SYSTEMD_OVERRIDE_FILE: &str =
    "/etc/systemd/system/nymvpnd.service.d/override.conf";

/// Systemd override body for guest `nym-vpnd` log verbosity (`-v` / `-vv`).
///
/// Matches `ssh-setup.sh` ExecStart shape: `run-as-service --disable-client-verification`.
pub(crate) fn daemon_log_level_override_content(
    daemon_bin: &str,
    verbosity_level: Verbosity,
) -> String {
    let verbosity = match verbosity_level {
        Verbosity::Info => "",
        Verbosity::Debug => "-v",
        Verbosity::Trace => "-vv",
    };
    let verbosity_arg = if verbosity.is_empty() {
        String::new()
    } else {
        format!(" {verbosity}")
    };
    format!(
        "[Service]\nExecStart=\nExecStart={daemon_bin} run-as-service --disable-client-verification{verbosity_arg}\n"
    )
}

#[cfg(unix)]
pub fn reboot() -> Result<(), test_rpc::Error> {
    log::debug!("Rebooting system");

    std::thread::spawn(|| {
        #[cfg(target_os = "linux")]
        let mut cmd = std::process::Command::new("/usr/sbin/shutdown");
        #[cfg(target_os = "macos")]
        let mut cmd = std::process::Command::new("/sbin/shutdown");
        cmd.args(["-r", "now"]);

        std::thread::sleep(std::time::Duration::from_secs(5));

        let _ = cmd.spawn().map_err(|error| {
            log::error!("Failed to spawn shutdown command: {error}");
            error
        });
    });

    Ok(())
}

#[cfg(target_os = "linux")]
pub async fn set_daemon_log_level(
    verbosity_level: Verbosity,
    service_name: &str,
    systemd_override_file: &str,
) -> Result<(), test_rpc::Error> {
    use tokio::io::AsyncWriteExt;
    log::debug!("Setting log level");

    let systemd_service_file_content =
        daemon_log_level_override_content(crate::app_nymvpn::NYMVPND_CLI_BIN, verbosity_level);

    let override_path = std::path::Path::new(systemd_override_file);
    if let Some(parent) = override_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| test_rpc::Error::ServiceChange(e.to_string()))?;
    }

    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(override_path)
        .await
        .map_err(|e| test_rpc::Error::ServiceChange(e.to_string()))?;

    file.write_all(systemd_service_file_content.as_bytes())
        .await
        .map_err(|e| test_rpc::Error::ServiceChange(e.to_string()))?;

    tokio::process::Command::new("systemctl")
        .args(["daemon-reload"])
        .status()
        .await
        .map_err(|e| test_rpc::Error::ServiceStart(e.to_string()))?;

    restart_app(service_name).await?;
    Ok(())
}

/// Restart the Mullvad VPN application.
///
/// This function waits for the app to successfully start again.
#[cfg(target_os = "linux")]
pub async fn restart_app(service_name: &str) -> Result<(), test_rpc::Error> {
    tokio::process::Command::new("systemctl")
        .args(["restart", service_name])
        .status()
        .await
        .map_err(|e| test_rpc::Error::ServiceStart(e.to_string()))?;
    wait_for_service_state(ServiceState::Running, service_name).await?;
    Ok(())
}

/// Stop the Mullvad VPN application.
///
/// This function waits for the app to successfully shut down.
#[cfg(target_os = "linux")]
pub async fn stop_app(service_name: &str) -> Result<(), test_rpc::Error> {
    tokio::process::Command::new("systemctl")
        .args(["stop", service_name])
        .status()
        .await
        .map_err(|e| test_rpc::Error::ServiceStop(e.to_string()))?;
    wait_for_service_state(ServiceState::Inactive, service_name).await?;

    Ok(())
}

/// Start the Mullvad VPN application.
///
/// This function waits for the app to successfully start again.
#[cfg(target_os = "linux")]
pub async fn start_app(service_name: &str) -> Result<(), test_rpc::Error> {
    tokio::process::Command::new("systemctl")
        .args(["start", service_name])
        .status()
        .await
        .map_err(|e| test_rpc::Error::ServiceStart(e.to_string()))?;
    wait_for_service_state(ServiceState::Running, service_name).await?;
    Ok(())
}

#[cfg(target_os = "linux")]
#[derive(Debug)]
struct EnvVar {
    var: String,
    value: String,
}

#[cfg(target_os = "linux")]
impl EnvVar {
    fn from_systemd_string(s: &str) -> Result<Self, &'static str> {
        // Here, we are only concerned with parsing a line that starts with "Environment".
        let error = "Failed to parse systemd env-config";
        let mut input = s.trim().split('=');
        let pre = input.next().ok_or(error)?;
        match pre {
            "Environment" => {
                // Process the input just a bit more - remove the leading and trailing quote (").
                let var = input
                    .next()
                    .ok_or(error)?
                    .trim_start_matches('"')
                    .to_string();
                let value = input.next().ok_or(error)?.trim_end_matches('"').to_string();
                Ok(EnvVar { var, value })
            }
            _ => Err(error),
        }
    }

    fn to_systemd_string(&self) -> String {
        format!(
            "Environment=\"{key}={value}\"",
            key = self.var,
            value = self.value
        )
    }
}

#[cfg(target_os = "linux")]
pub async fn set_daemon_environment(
    env: HashMap<String, String>,
    service_name: &str,
    systemd_override_file: &str,
) -> Result<(), test_rpc::Error> {
    use std::{fmt::Write, ops::Not};

    let mut override_content = String::new();
    override_content.push_str("[Service]\n");

    for env_var in env
        .into_iter()
        .map(|(var, value)| EnvVar { var, value })
        .map(|env_var| env_var.to_systemd_string())
    {
        writeln!(&mut override_content, "{env_var}")
            .map_err(|err| test_rpc::Error::ServiceChange(err.to_string()))?;
    }

    let override_path = std::path::Path::new(systemd_override_file);
    if let Some(parent) = override_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| test_rpc::Error::ServiceChange(e.to_string()))?;
    }

    tokio::fs::write(override_path, override_content)
        .await
        .map_err(|e| test_rpc::Error::ServiceChange(e.to_string()))?;

    if tokio::process::Command::new("systemctl")
        .args(["daemon-reload"])
        .status()
        .await
        .map_err(|e| test_rpc::Error::Io(e.to_string()))?
        .success()
        .not()
    {
        return Err(test_rpc::Error::ServiceChange(
            "Daemon service could not be reloaded".to_owned(),
        ));
    };

    if tokio::process::Command::new("systemctl")
        .args(["restart", service_name])
        .status()
        .await
        .map_err(|e| test_rpc::Error::Io(e.to_string()))?
        .success()
        .not()
    {
        return Err(test_rpc::Error::ServiceStart(
            "Daemon service could not be restarted".to_owned(),
        ));
    };

    wait_for_service_state(ServiceState::Running, service_name).await?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn reboot() -> Result<(), test_rpc::Error> {
    log::debug!("Rebooting system");

    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_secs(5));
        let _ = std::process::Command::new("shutdown")
            .args(["/r", "/t", "0"])
            .spawn()
            .map_err(|error| {
                log::error!("Failed to spawn shutdown command: {error}");
                error
            });
    });

    Ok(())
}

#[cfg(target_os = "windows")]
pub fn get_system_path_var() -> Result<String, test_rpc::Error> {
    std::env::var("PATH").map_err(|_| test_rpc::Error::Syscall)
}

#[cfg(target_os = "windows")]
pub async fn disable_system_service_startup() -> Result<(), test_rpc::Error> {
    // TODO: Implement Windows service disable
    log::error!("disable_system_service_startup is not yet implemented on Windows");
    Err(test_rpc::Error::Syscall)
}

#[cfg(target_os = "windows")]
pub async fn enable_system_service_startup() -> Result<(), test_rpc::Error> {
    // TODO: Implement Windows service enable
    log::error!("enable_system_service_startup is not yet implemented on Windows");
    Err(test_rpc::Error::Syscall)
}

#[cfg(target_os = "windows")]
#[allow(dead_code)]
pub fn get_os_version() -> Result<test_rpc::meta::OsVersion, test_rpc::Error> {
    // TODO: Return proper Windows version
    Err(test_rpc::Error::Syscall)
}

#[cfg(target_os = "linux")]
pub async fn get_daemon_environment(
    systemd_override_file: &str,
) -> Result<HashMap<String, String>, test_rpc::Error> {
    let text = tokio::fs::read_to_string(systemd_override_file)
        .await
        .map_err(|err| test_rpc::Error::FileSystem(err.to_string()))?;

    let env: HashMap<String, String> = parse_systemd_env_file(&text)
        .map(|EnvVar { var, value }| (var, value))
        .collect();
    Ok(env)
}

/// Parse a systemd env-file. `input` is assumed to be the entire text content of a systemd-env
/// file.
///
/// Example systemd-env file:
/// ```
/// [Service]
/// Environment="VAR1=pGNqduRFkB4K9C2vijOmUDa2kPtUhArN"
/// Environment="VAR2=JP8YLOc2bsNlrGuD6LVTq7L36obpjzxd"
/// ```
#[cfg(target_os = "linux")]
fn parse_systemd_env_file(input: &str) -> impl Iterator<Item = EnvVar> + '_ {
    input
        .lines()
        .map(EnvVar::from_systemd_string)
        .filter_map(|env_var| env_var.ok())
        .inspect(|env_var| log::trace!("Parsed {env_var:?}"))
}

#[cfg(target_os = "linux")]
enum ServiceState {
    Running,
    Inactive,
}

#[cfg(target_os = "linux")]
async fn wait_for_service_state(
    awaited_state: ServiceState,
    service_name: &str,
) -> Result<(), test_rpc::Error> {
    const RETRY_ATTEMPTS: usize = 10;
    let mut attempt = 0;
    loop {
        attempt += 1;
        if attempt > RETRY_ATTEMPTS {
            return Err(test_rpc::Error::ServiceStart(String::from(
                "Awaiting new service state timed out",
            )));
        }

        let output = tokio::process::Command::new("systemctl")
            .args(["status", service_name])
            .output()
            .await
            .map_err(|e| test_rpc::Error::ServiceNotFound(e.to_string()))?
            .stdout;
        let output = String::from_utf8_lossy(&output);

        match awaited_state {
            ServiceState::Running => {
                if output.contains("active (running)") {
                    break;
                }
            }
            ServiceState::Inactive => {
                if output.contains("inactive (dead)") {
                    break;
                }
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    }
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn get_os_version() -> Result<test_rpc::meta::OsVersion, test_rpc::Error> {
    Ok(test_rpc::meta::OsVersion::Linux)
}

#[cfg(target_os = "macos")]
pub fn get_os_version() -> Result<test_rpc::meta::OsVersion, test_rpc::Error> {
    // Get macOS major version via sysctl
    let output = std::process::Command::new("sw_vers")
        .arg("-productVersion")
        .output()
        .map_err(|_| test_rpc::Error::Syscall)?;
    let version_str = String::from_utf8_lossy(&output.stdout);
    let major: u32 = version_str
        .trim()
        .split('.')
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    Ok(test_rpc::meta::OsVersion::Macos(
        test_rpc::meta::MacosVersion { major },
    ))
}

#[cfg(test)]
mod test {
    use super::{NYM_VPN_SYSTEMD_OVERRIDE_FILE, daemon_log_level_override_content};
    use test_rpc::nym_daemon::Verbosity;

    #[cfg(target_os = "linux")]
    #[test]
    fn parse_systemd_environment_variables() {
        use super::parse_systemd_env_file;
        // Define an example systemd environment file
        let systemd_file = "
        [Service]
        Environment=\"var1=value1\"
        Environment=\"var2=value2\"
        ";

        // Parse the "file"
        let env_vars: Vec<_> = parse_systemd_env_file(systemd_file).collect();

        // Assert that the environment variables it defines are parsed as expected.
        assert_eq!(env_vars.len(), 2);
        let first = env_vars.first().unwrap();
        assert_eq!(first.var, "var1");
        assert_eq!(first.value, "value1");
        let second = env_vars.get(1).unwrap();
        assert_eq!(second.var, "var2");
        assert_eq!(second.value, "value2");
    }

    #[test]
    fn systemd_override_path_matches_nymvpnd_unit() {
        assert!(
            NYM_VPN_SYSTEMD_OVERRIDE_FILE.contains("nymvpnd.service.d"),
            "drop-in dir must match unit nymvpnd.service, got {NYM_VPN_SYSTEMD_OVERRIDE_FILE}"
        );
        assert!(
            !NYM_VPN_SYSTEMD_OVERRIDE_FILE.contains("nymvpn.service.d"),
            "Mullvad leftover nymvpn.service.d would never apply"
        );
    }

    #[test]
    fn daemon_log_level_override_is_nym_shaped() {
        let bin = "/opt/testing/nym-vpnd";
        let trace = daemon_log_level_override_content(bin, Verbosity::Trace);
        assert!(trace.contains(
            "ExecStart=/opt/testing/nym-vpnd run-as-service --disable-client-verification -vv"
        ));
        assert!(!trace.contains("disable-stdout-timestamps"));
        assert!(!trace.contains("/usr/bin/nymvpnd.service"));

        let info = daemon_log_level_override_content(bin, Verbosity::Info);
        assert!(info.contains(
            "ExecStart=/opt/testing/nym-vpnd run-as-service --disable-client-verification\n"
        ));
        assert!(!info.contains(" -v"));

        let debug = daemon_log_level_override_content(bin, Verbosity::Debug);
        assert!(debug.contains(
            "ExecStart=/opt/testing/nym-vpnd run-as-service --disable-client-verification -v\n"
        ));
    }
}
