// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    cmp::Ordering,
    fmt::{self, Formatter},
    io,
    str::FromStr,
};

use objc2_foundation::NSProcessInfo;

#[derive(Debug, Clone)]
pub struct AppleVersion {
    raw_version: String,
    major: u32,
    minor: u32,
    patch: Option<u32>,
}

impl PartialEq for AppleVersion {
    fn eq(&self, other: &Self) -> bool {
        self.major_version() == other.major_version()
            && self.minor_version() == other.minor_version()
            && self.patch_version() == other.patch_version()
    }
}

impl PartialOrd for AppleVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let major = self.major_version().partial_cmp(&other.major_version())?;
        let minor = self.minor_version().partial_cmp(&other.minor_version())?;
        let patch = self.patch_version().partial_cmp(&other.patch_version())?;
        Some(major.then(minor).then(patch))
    }
}

impl fmt::Display for AppleVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.version())
    }
}

impl FromStr for AppleVersion {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (major, minor, patch) = parse_version_output(s).ok_or(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Failed to parse raw version string",
        ))?;
        Ok(AppleVersion {
            raw_version: s.to_owned(),
            major,
            minor,
            patch,
        })
    }
}

impl AppleVersion {
    pub fn current() -> AppleVersion {
        let version = NSProcessInfo::processInfo().operatingSystemVersion();

        Self {
            raw_version: format!(
                "{}.{}.{}",
                version.majorVersion, version.minorVersion, version.patchVersion
            ),
            major: version.majorVersion as u32,
            minor: version.minorVersion as u32,
            patch: Some(version.patchVersion as u32),
        }
    }

    /// Return the current version as a string (e.g. 14.2.1)
    pub fn version(&self) -> String {
        self.raw_version.clone()
    }

    /// Return the current version as a string (e.g. 14.2), not including the patch version
    pub fn short_version(&self) -> String {
        format!("{}.{}", self.major_version(), self.minor_version())
    }

    pub fn major_version(&self) -> u32 {
        self.major
    }

    pub fn minor_version(&self) -> u32 {
        self.minor
    }

    pub fn patch_version(&self) -> u32 {
        self.patch.unwrap_or(0)
    }
}

fn parse_version_output(output: &str) -> Option<(u32, u32, Option<u32>)> {
    let mut parts = output.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next().and_then(|patch| patch.parse().ok());
    Some((major, minor, patch))
}

#[test]
fn test_get_current_version() {
    AppleVersion::current().version();
}

#[test]
fn test_version_parsing() {
    let version = AppleVersion::from_str("14.2.1").expect("failed to parse version");
    assert_eq!(version.major_version(), 14);
    assert_eq!(version.minor_version(), 2);
    assert_eq!(version.patch_version(), 1);
}

#[test]
fn test_version_order() {
    assert_eq!(
        AppleVersion::from_str("14.0").unwrap(),
        AppleVersion::from_str("14.0.0").unwrap()
    );

    assert_eq!(
        AppleVersion::from_str("14.0")
            .unwrap()
            .partial_cmp(&AppleVersion::from_str("14.0.0").unwrap()),
        Some(Ordering::Equal),
    );

    // test major version
    assert!(AppleVersion::from_str("14.0").unwrap() < AppleVersion::from_str("15.2.1").unwrap());
    assert!(AppleVersion::from_str("14.0").unwrap() > AppleVersion::from_str("12.1").unwrap());

    // test minor version
    assert!(AppleVersion::from_str("14.3").unwrap() > AppleVersion::from_str("14.2").unwrap());
    assert!(AppleVersion::from_str("14.2").unwrap() < AppleVersion::from_str("14.3").unwrap());

    // test patch version
    assert!(AppleVersion::from_str("14.2.1").unwrap() > AppleVersion::from_str("14.2").unwrap());
    assert!(AppleVersion::from_str("14.2.2").unwrap() > AppleVersion::from_str("14.2.1").unwrap());
    assert!(AppleVersion::from_str("14.2.2").unwrap() < AppleVersion::from_str("14.2.3").unwrap());
}
