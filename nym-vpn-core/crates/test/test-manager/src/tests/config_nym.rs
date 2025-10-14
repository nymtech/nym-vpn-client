// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{net::Ipv4Addr, ops::Deref, path::Path, sync::OnceLock};
use test_rpc::meta::Os;

pub static TEST_CONFIG_NYM: TestConfigContainer = TestConfigContainer::new();

/// Script for bootstrapping the test-runner after the test-manager has successfully logged in.
pub const BOOTSTRAP_SCRIPT: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../scripts/",
    "ssh-setup.sh"
));

/// Constants that are accessible from each test via `TEST_CONFIG`.
/// The constants must be initialized before running any tests using `TEST_CONFIG.init()`.
#[derive(Debug, Clone)]
pub struct TestConfigNym {
    pub mnemonic: String,

    pub artifacts_dir: String,
    pub app_package_filename: String,
    pub app_package_to_upgrade_from_filename: Option<String>,
    pub ui_e2e_tests_filename: Option<String>,

    pub host_bridge_name: String,
    pub host_bridge_ip: Ipv4Addr,
    pub os: Os,
}

impl TestConfigNym {
    #[allow(clippy::too_many_arguments)]
    // TODO: This argument list is very long, we should strive to shorten it if possible.
    pub const fn new(
        mnemonic: String,
        artifacts_dir: String,
        app_package_filename: String,
        app_package_to_upgrade_from_filename: Option<String>,
        ui_e2e_tests_filename: Option<String>,
        host_bridge_name: String,
        host_bridge_ip: Ipv4Addr,
        os: Os,
    ) -> Self {
        Self {
            mnemonic,
            artifacts_dir,
            app_package_filename,
            app_package_to_upgrade_from_filename,
            ui_e2e_tests_filename,
            host_bridge_name,
            host_bridge_ip,
            os,
        }
    }
}

/// The OpenVPN CA certificate to use with the installed Mullvad App.
#[derive(Clone, Debug)]
pub struct OpenVPNCertificate(Vec<u8>);

impl OpenVPNCertificate {
    pub fn from_file(path: impl AsRef<Path>) -> std::io::Result<Self> {
        Ok(Self(std::fs::read(path)?))
    }
}

/// A script which should be run *in* the test runner before the test run begins.
#[derive(Clone, Debug)]
pub struct BootstrapScript(Vec<u8>);

impl Deref for BootstrapScript {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for BootstrapScript {
    fn default() -> Self {
        Self(Vec::from(BOOTSTRAP_SCRIPT))
    }
}

#[derive(Debug, Clone)]
pub struct TestConfigContainer(OnceLock<TestConfigNym>);

impl TestConfigContainer {
    const fn new() -> Self {
        TestConfigContainer(OnceLock::new())
    }

    /// Initializes the constants.
    ///
    /// # Panics
    ///
    /// This panics if the config has already been initialized.
    pub fn init(&self, inner: TestConfigNym) {
        self.0.set(inner).unwrap()
    }
}

impl Deref for TestConfigContainer {
    type Target = TestConfigNym;

    fn deref(&self) -> &Self::Target {
        self.0.get().unwrap()
    }
}
