// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod v9 {
    use std::collections::HashSet;

    use serde::{Deserialize, Serialize};

    use crate::service::error::GeoExclusionConfigError;

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct GeoExclusionSettings {
        pub enabled: bool,
        pub listen_port: u16,
        pub excluded_countries: Vec<String>,
    }

    /// Countries for which we ship IP-range and domain data (see nym-socks5-proxy's
    /// builtin/download_sources.py COUNTRY_CODES, which must be kept in sync with this).
    pub const SUPPORTED_COUNTRIES: &[&str] = &["CN", "RU"];

    /// Shared validation for excluded-country lists, used both when a caller sets new
    /// excluded countries at runtime and when a persisted config is loaded from disk -
    /// so a manually edited or legacy config can't smuggle an invalid/unsupported/
    /// duplicate country code past validation.
    pub fn validate_excluded_countries(
        excluded_countries: &[String],
    ) -> Result<(), GeoExclusionConfigError> {
        let mut seen = HashSet::with_capacity(excluded_countries.len());
        for country in excluded_countries {
            if country.len() != 2 || !country.chars().all(|c| c.is_ascii_uppercase()) {
                return Err(GeoExclusionConfigError::InvalidCountryCode(country.clone()));
            } else if !SUPPORTED_COUNTRIES.contains(&country.as_str()) {
                return Err(GeoExclusionConfigError::UnsupportedCountry(
                    country.clone(),
                    SUPPORTED_COUNTRIES.join(", "),
                ));
            } else if !seen.insert(country.as_str()) {
                return Err(GeoExclusionConfigError::DuplicateCountry(country.clone()));
            }
        }
        Ok(())
    }

    /// Port 1080 is reserved for the mixnet socks5 proxy.
    const RESERVED_PORT: u16 = 1080;

    /// Shared validation for the geo-exclusion listen port, used both when a caller sets
    /// a new port at runtime and when a persisted config is loaded from disk.
    pub fn validate_listen_port(listen_port: u16) -> Result<(), GeoExclusionConfigError> {
        if listen_port == RESERVED_PORT {
            Err(GeoExclusionConfigError::ReservedPort(listen_port))
        } else if listen_port == 0 {
            Err(GeoExclusionConfigError::InvalidPort)
        } else {
            Ok(())
        }
    }

    impl TryFrom<GeoExclusionSettings> for nym_vpn_lib_types::GeoExclusionSettings {
        type Error = GeoExclusionConfigError;

        fn try_from(value: GeoExclusionSettings) -> Result<Self, Self::Error> {
            validate_listen_port(value.listen_port)?;
            validate_excluded_countries(&value.excluded_countries)?;
            Ok(Self {
                enabled: value.enabled,
                listen_port: value.listen_port,
                excluded_countries: value.excluded_countries,
            })
        }
    }

    impl From<&nym_vpn_lib_types::GeoExclusionSettings> for GeoExclusionSettings {
        fn from(value: &nym_vpn_lib_types::GeoExclusionSettings) -> Self {
            Self {
                enabled: value.enabled,
                listen_port: value.listen_port,
                excluded_countries: value.excluded_countries.clone(),
            }
        }
    }
}
