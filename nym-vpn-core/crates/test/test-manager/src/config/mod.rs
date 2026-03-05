// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Test manager configuration.

mod error;
mod io;
mod manifest;
mod vm;

use error::Error;
pub use io::ConfigFile;
pub use manifest::{Config, Display};
pub use vm::{OsType, PackageType, Provisioner, VmConfig, VmType};
