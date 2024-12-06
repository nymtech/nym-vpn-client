// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    str::FromStr,
};

use ipnetwork::{Ipv4Network, Ipv6Network};

use crate::UniffiCustomTypeConverter;

uniffi::custom_type!(Ipv4Addr, String);
uniffi::custom_type!(Ipv6Addr, String);
uniffi::custom_type!(IpAddr, String);
uniffi::custom_type!(Ipv4Network, String);
uniffi::custom_type!(Ipv6Network, String);

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

impl UniffiCustomTypeConverter for IpAddr {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(IpAddr::from_str(&val)?)
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.to_string()
    }
}

impl UniffiCustomTypeConverter for Ipv4Network {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(Ipv4Network::from_str(&val)?)
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.to_string()
    }
}

impl UniffiCustomTypeConverter for Ipv6Network {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(Ipv6Network::from_str(&val)?)
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.to_string()
    }
}
