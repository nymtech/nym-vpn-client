// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{net::Ipv4Addr, ops::Deref, sync::OnceLock};
use test_rpc::meta::Os;

pub static TEST_CONFIG_NYM: TestConfigContainer = TestConfigContainer::new();

/// Script for bootstrapping the test-runner after the test-manager has successfully logged in.
pub const BOOTSTRAP_SCRIPT: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../scripts/",
    "ssh-setup.sh"
));

/// Blocking scripts executed inside the guest VM during circumvention tests. Baked into the
/// binary so the test-manager has no filesystem dependency on the source tree at runtime.
pub const IP_BLOCK_SCRIPT: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../scripts/blocking/ip_block.sh"
));
pub const SNI_BLOCK_SCRIPT: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../scripts/blocking/sni_block.sh"
));
pub const DELAYED_IP_BLOCK_SCRIPT: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../scripts/blocking/delayed_ip_block.sh"
));
pub const UDP_BLOCK_SCRIPT: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../scripts/blocking/udp_block.sh"
));

/// Constants that are accessible from each test via `TEST_CONFIG`.
/// The constants must be initialized before running any tests using `TEST_CONFIG.init()`.
#[derive(Debug, Clone)]
pub struct TestConfigNym {
    pub mnemonic: String,
    pub _artifacts_dir: String,
    pub _host_bridge_name: String,
    pub _host_bridge_ip: Ipv4Addr,
    pub os: Os,
}

impl TestConfigNym {
    #[allow(clippy::too_many_arguments)]
    // TODO: This argument list is very long, we should strive to shorten it if possible.
    pub const fn new(
        mnemonic: String,
        artifacts_dir: String,
        host_bridge_name: String,
        host_bridge_ip: Ipv4Addr,
        os: Os,
    ) -> Self {
        Self {
            mnemonic,
            _artifacts_dir: artifacts_dir,
            _host_bridge_name: host_bridge_name,
            _host_bridge_ip: host_bridge_ip,
            os,
        }
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
