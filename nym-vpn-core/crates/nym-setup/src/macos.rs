// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Context, Result, anyhow};
use serde::Serialize;
use std::{
    collections::HashMap,
    process::{Child, Command, Stdio},
};

const DAEMON_PLIST_PATH: &str = "/Library/LaunchDaemons/net.nymtech.vpn.daemon.plist";
const DAEMON_BUNDLE_IDENTIFIER: &str = "net.nymtech.vpn.daemon";
const APP_BUNDLE_IDENTIFIER: &str = "net.nymtech.vpn";

pub fn install_service() -> Result<()> {
    let binary_path = crate::service_binary::service_binary_path()?
        .into_os_string()
        .into_string()
        .map_err(|s| anyhow!("Failed to convert binary path to string: {s:?}"))?;

    let launch_daemon_config = LaunchDaemonConfig {
        label: DAEMON_BUNDLE_IDENTIFIER.to_owned(),
        program_arguments: vec![binary_path, "-v".to_owned(), "run-as-service".to_owned()],
        mach_services: HashMap::from([(DAEMON_BUNDLE_IDENTIFIER.to_owned(), true)]),
        environment_variables: Default::default(),
        user_name: "root".to_owned(),
        run_at_load: true,
        keep_alive: true,
        soft_resource_limits: Some(SoftResourceLimits {
            number_of_files: Some(1024),
        }),
        associated_bundle_identifiers: vec![APP_BUNDLE_IDENTIFIER.to_owned()],
        standard_error_path: None,
        standard_output_path: None,
    };

    if let Err(e) = unload_launch_daemon() {
        eprintln!("{e}");
    }

    plist::to_file_xml(DAEMON_PLIST_PATH, &launch_daemon_config)
        .with_context(|| "Failed to write daemon configuration")?;
    load_launch_daemon()
}

fn load_launch_daemon() -> Result<()> {
    let output = launchctl(LaunchctlAction::LoadService)?.wait_with_output()?;
    if output.stderr.is_empty() {
        Ok(())
    } else {
        Err(anyhow!("Launchd failed to load config"))
    }
}

fn unload_launch_daemon() -> Result<()> {
    let output = launchctl(LaunchctlAction::UnloadService)?.wait_with_output()?;
    if output.stderr.is_empty() {
        Ok(())
    } else {
        Err(anyhow!("Launchd failed to unload config"))
    }
}

enum LaunchctlAction {
    LoadService,
    UnloadService,
}

impl LaunchctlAction {
    fn to_str(&self) -> &'static str {
        match self {
            Self::LoadService => "load",
            Self::UnloadService => "unload",
        }
    }
}

fn launchctl(action: LaunchctlAction) -> std::io::Result<Child> {
    Command::new("launchctl")
        .arg(action.to_str())
        .arg("-w")
        .arg(DAEMON_PLIST_PATH)
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct LaunchDaemonConfig {
    label: String,
    program_arguments: Vec<String>,
    mach_services: HashMap<String, bool>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    environment_variables: HashMap<String, String>,
    user_name: String,
    run_at_load: bool,
    keep_alive: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    soft_resource_limits: Option<SoftResourceLimits>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    associated_bundle_identifiers: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    standard_error_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    standard_output_path: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct SoftResourceLimits {
    #[serde(skip_serializing_if = "Option::is_none")]
    number_of_files: Option<u32>,
}
