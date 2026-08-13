// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::{EntryPoint, ExitPoint, FrontingMode, Profile, TunnelType};

pub struct ProfileSpecifics {
    pub entry_point: EntryPoint,
    pub exit_point: ExitPoint,
    pub tunnel_type: TunnelType,
    pub fronting_mode: FrontingMode,
}

impl From<Profile> for ProfileSpecifics {
    fn from(profile: Profile) -> Self {
        let (entry_point, exit_point, tunnel_type, fronting_mode) = match profile {
            Profile::Safest => (
                EntryPoint::Auto {
                    exclude_user_country: true,
                },
                ExitPoint::Auto {
                    exclude_entry_point_country: true,
                    exclude_user_country: true,
                },
                TunnelType::Wireguard,
                FrontingMode::Always,
            ),
            Profile::MostPrivate => (
                EntryPoint::Auto {
                    exclude_user_country: true,
                },
                ExitPoint::Auto {
                    exclude_entry_point_country: true,
                    exclude_user_country: true,
                },
                TunnelType::Mixnet,
                FrontingMode::OnRetry,
            ),
            Profile::Fastest => (
                EntryPoint::Auto {
                    exclude_user_country: false,
                },
                ExitPoint::Auto {
                    exclude_entry_point_country: false,
                    exclude_user_country: false,
                },
                TunnelType::Wireguard,
                FrontingMode::OnRetry,
            ),
            Profile::Random => (
                EntryPoint::Random,
                ExitPoint::Random,
                TunnelType::Wireguard,
                FrontingMode::OnRetry,
            ),
        };
        Self {
            entry_point,
            exit_point,
            tunnel_type,
            fronting_mode,
        }
    }
}
