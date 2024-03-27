// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use crate::platform::error::FFIError;
use crate::{NodeIdentity, Recipient, UniffiCustomTypeConverter};
use ipnetwork::IpNetwork;
use nym_explorer_client::Location;
use nym_gateway_directory::{EntryPoint, ExitPoint};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::str::FromStr;
use talpid_types::net::wireguard::{PresharedKey, PrivateKey, PublicKey};
use url::Url;

uniffi::custom_type!(Ipv4Addr, String);
uniffi::custom_type!(Ipv6Addr, String);
uniffi::custom_type!(IpAddr, String);
uniffi::custom_type!(PrivateKey, String);
uniffi::custom_type!(PublicKey, String);
uniffi::custom_type!(IpNetwork, String);
uniffi::custom_type!(SocketAddr, String);
uniffi::custom_type!(PresharedKey, String);
uniffi::custom_type!(Url, String);
uniffi::custom_type!(NodeIdentity, String);
uniffi::custom_type!(Recipient, String);

impl UniffiCustomTypeConverter for NodeIdentity {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(NodeIdentity::from_base58_string(val)?)
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.to_base58_string()
    }
}

impl UniffiCustomTypeConverter for Recipient {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(Recipient::try_from_base58_string(val)?)
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.to_string()
    }
}

impl UniffiCustomTypeConverter for Url {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(Url::from_str(&val)?)
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.to_string()
    }
}

impl UniffiCustomTypeConverter for PrivateKey {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(PrivateKey::from(
            *PublicKey::from_base64(&val)
                .map_err(|_| FFIError::InvalidValueUniffi)?
                .as_bytes(),
        ))
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.to_base64()
    }
}

impl UniffiCustomTypeConverter for PublicKey {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(PublicKey::from_base64(&val).map_err(|_| FFIError::InvalidValueUniffi)?)
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.to_base64()
    }
}

impl UniffiCustomTypeConverter for IpAddr {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(IpAddr::from_str(&val)?)
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.to_string()
    }
}

impl UniffiCustomTypeConverter for Ipv4Addr {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(Ipv4Addr::from_str(&val)?)
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.to_string()
    }
}

impl UniffiCustomTypeConverter for Ipv6Addr {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(Ipv6Addr::from_str(&val)?)
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.to_string()
    }
}

impl UniffiCustomTypeConverter for IpNetwork {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(IpNetwork::from_str(&val)?)
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.to_string()
    }
}

impl UniffiCustomTypeConverter for SocketAddr {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(SocketAddr::from_str(&val)?)
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.to_string()
    }
}

impl UniffiCustomTypeConverter for PresharedKey {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(PresharedKey::from(Box::new(
            PrivateKey::into_custom(val)?.to_bytes(),
        )))
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        PrivateKey::from_custom(PrivateKey::from(*obj.as_bytes()))
    }
}

#[derive(uniffi::Record)]
pub struct FfiLocation {
    pub two_letter_iso_country_code: String,
    pub three_letter_iso_country_code: String,
    pub country_name: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

impl From<Location> for FfiLocation {
    fn from(value: Location) -> Self {
        FfiLocation {
            two_letter_iso_country_code: value.two_letter_iso_country_code,
            three_letter_iso_country_code: value.three_letter_iso_country_code,
            country_name: value.country_name,
            latitude: value.latitude,
            longitude: value.longitude,
        }
    }
}

#[derive(uniffi::Enum)]
pub enum FfiEntryPoint {
    Gateway { identity: NodeIdentity },
    Location { location: String },
    RandomLowLatency,
    Random,
}

impl From<FfiEntryPoint> for EntryPoint {
    fn from(value: FfiEntryPoint) -> Self {
        match value {
            FfiEntryPoint::Gateway { identity } => EntryPoint::Gateway { identity },
            FfiEntryPoint::Location { location } => EntryPoint::Location { location },
            FfiEntryPoint::RandomLowLatency => EntryPoint::RandomLowLatency,
            FfiEntryPoint::Random => EntryPoint::Random,
        }
    }
}

#[derive(uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
pub enum FfiExitPoint {
    Address { address: Recipient },
    Gateway { identity: NodeIdentity },
    Location { location: String },
}

impl From<FfiExitPoint> for ExitPoint {
    fn from(value: FfiExitPoint) -> Self {
        match value {
            FfiExitPoint::Address { address } => ExitPoint::Address { address },
            FfiExitPoint::Gateway { identity } => ExitPoint::Gateway { identity },
            FfiExitPoint::Location { location } => ExitPoint::Location { location },
        }
    }
}
