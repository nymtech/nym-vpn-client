// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::{Profile, ProfileOptions};

use crate::{conversions::ConversionError, proto};

impl From<proto::Profile> for Profile {
    fn from(proto_profile: proto::Profile) -> Self {
        match proto_profile {
            proto::Profile::Safest => Self::Safest,
            proto::Profile::MostPrivate => Self::MostPrivate,
            proto::Profile::Fastest => Self::Fastest,
            proto::Profile::Random => Self::Random,
        }
    }
}

impl From<Profile> for proto::Profile {
    fn from(profile: Profile) -> Self {
        match profile {
            Profile::Safest => Self::Safest,
            Profile::MostPrivate => Self::MostPrivate,
            Profile::Fastest => Self::Fastest,
            Profile::Random => Self::Random,
        }
    }
}

impl TryFrom<proto::ProfileOptions> for ProfileOptions {
    type Error = ConversionError;

    fn try_from(value: proto::ProfileOptions) -> Result<Self, Self::Error> {
        let proto_profile = proto::Profile::try_from(value.profile)
            .map_err(|err| ConversionError::Decode("ProfileOptions.profile", err))?;
        let profile = Profile::from(proto_profile);

        Ok(Self { profile })
    }
}

impl TryFrom<ProfileOptions> for proto::ProfileOptions {
    type Error = ConversionError;

    fn try_from(value: ProfileOptions) -> Result<Self, Self::Error> {
        let proto_profile = proto::Profile::from(value.profile);
        Ok(Self {
            profile: proto_profile as i32,
        })
    }
}
