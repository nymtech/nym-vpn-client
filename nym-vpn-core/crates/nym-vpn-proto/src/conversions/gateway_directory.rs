// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{conversions::ConversionError, proto};

impl From<nym_vpn_lib_types::GatewayType> for proto::GatewayType {
    fn from(value: nym_vpn_lib_types::GatewayType) -> Self {
        match value {
            nym_vpn_lib_types::GatewayType::MixnetEntry => proto::GatewayType::MixnetEntry,
            nym_vpn_lib_types::GatewayType::MixnetExit => proto::GatewayType::MixnetExit,
            nym_vpn_lib_types::GatewayType::Wg => proto::GatewayType::Wg,
        }
    }
}

impl From<proto::GatewayType> for nym_vpn_lib_types::GatewayType {
    fn from(value: proto::GatewayType) -> Self {
        match value {
            proto::GatewayType::MixnetEntry => nym_vpn_lib_types::GatewayType::MixnetEntry,
            proto::GatewayType::MixnetExit => nym_vpn_lib_types::GatewayType::MixnetExit,
            proto::GatewayType::Wg => nym_vpn_lib_types::GatewayType::Wg,
        }
    }
}

impl TryFrom<proto::EntryNode> for nym_vpn_lib_types::EntryPoint {
    type Error = ConversionError;

    fn try_from(value: proto::EntryNode) -> Result<Self, Self::Error> {
        let entry_enum_value = value
            .entry_node_enum
            .ok_or(ConversionError::NoValueSet("EntryNode.entry_node_enum"))?;

        Ok(match entry_enum_value {
            proto::entry_node::EntryNodeEnum::Gateway(gateway) => {
                let identity = nym_vpn_lib_types::NodeIdentity::from_base58_string(&gateway.id)
                    .map_err(|err| {
                        ConversionError::Generic(format!("failed to parse gateway id: {err}"))
                    })?;
                nym_vpn_lib_types::EntryPoint::Gateway { identity }
            }

            proto::entry_node::EntryNodeEnum::Country(country) => {
                nym_vpn_lib_types::EntryPoint::Country {
                    two_letter_iso_country_code: country.two_letter_iso_country_code.to_string(),
                }
            }
            proto::entry_node::EntryNodeEnum::Region(region) => {
                nym_vpn_lib_types::EntryPoint::Region {
                    region: region.region.to_string(),
                }
            }
            proto::entry_node::EntryNodeEnum::Random(_) => nym_vpn_lib_types::EntryPoint::Random,
            proto::entry_node::EntryNodeEnum::Auto(auto_entry) => {
                nym_vpn_lib_types::EntryPoint::Auto {
                    exclude_user_country: auto_entry.exclude_user_country,
                }
            }
        })
    }
}

impl From<nym_vpn_lib_types::EntryPoint> for proto::EntryNode {
    fn from(value: nym_vpn_lib_types::EntryPoint) -> Self {
        match value {
            nym_vpn_lib_types::EntryPoint::Gateway { identity } => proto::EntryNode {
                entry_node_enum: Some(proto::entry_node::EntryNodeEnum::Gateway(
                    proto::GatewayId {
                        id: identity.to_base58_string(),
                    },
                )),
            },
            nym_vpn_lib_types::EntryPoint::Country {
                two_letter_iso_country_code,
            } => proto::EntryNode {
                entry_node_enum: Some(proto::entry_node::EntryNodeEnum::Country(proto::Country {
                    two_letter_iso_country_code,
                })),
            },
            nym_vpn_lib_types::EntryPoint::Region { region } => proto::EntryNode {
                entry_node_enum: Some(proto::entry_node::EntryNodeEnum::Region(proto::Region {
                    region,
                })),
            },
            nym_vpn_lib_types::EntryPoint::Random => proto::EntryNode {
                entry_node_enum: Some(proto::entry_node::EntryNodeEnum::Random(())),
            },
            nym_vpn_lib_types::EntryPoint::Auto {
                exclude_user_country,
            } => proto::EntryNode {
                entry_node_enum: Some(proto::entry_node::EntryNodeEnum::Auto(proto::AutoEntry {
                    exclude_user_country,
                })),
            },
        }
    }
}

impl TryFrom<proto::ExitNode> for nym_vpn_lib_types::ExitPoint {
    type Error = ConversionError;

    fn try_from(value: proto::ExitNode) -> Result<Self, Self::Error> {
        let exit_enum_value = value
            .exit_node_enum
            .ok_or(ConversionError::NoValueSet("ExitNode.exit_node_enum"))?;

        Ok(match exit_enum_value {
            proto::exit_node::ExitNodeEnum::Address(address) => {
                let address = nym_vpn_lib_types::Recipient::try_from_base58_string(
                    address.nym_address.clone(),
                )
                .map_err(|err| {
                    ConversionError::Generic(format!("failed to parse exit node address: {err}"))
                })?;
                nym_vpn_lib_types::ExitPoint::Address {
                    address: Box::new(address),
                }
            }
            proto::exit_node::ExitNodeEnum::Gateway(gateway) => {
                let identity = nym_vpn_lib_types::NodeIdentity::from_base58_string(&gateway.id)
                    .map_err(|err| {
                        ConversionError::Generic(format!("failed to parse gateway id: {err}"))
                    })?;
                nym_vpn_lib_types::ExitPoint::Gateway { identity }
            }
            proto::exit_node::ExitNodeEnum::Country(country) => {
                nym_vpn_lib_types::ExitPoint::Country {
                    two_letter_iso_country_code: country.two_letter_iso_country_code,
                }
            }
            proto::exit_node::ExitNodeEnum::Region(region) => {
                nym_vpn_lib_types::ExitPoint::Region {
                    region: region.region,
                }
            }
            proto::exit_node::ExitNodeEnum::Random(_) => nym_vpn_lib_types::ExitPoint::Random,
            proto::exit_node::ExitNodeEnum::Auto(auto_exit) => nym_vpn_lib_types::ExitPoint::Auto {
                exclude_entry_point_country: auto_exit.exclude_entry_point_country,
                exclude_user_country: auto_exit.exclude_user_country,
            },
        })
    }
}

impl From<nym_vpn_lib_types::ExitPoint> for proto::ExitNode {
    fn from(value: nym_vpn_lib_types::ExitPoint) -> Self {
        let exit_node_enum = match value {
            nym_vpn_lib_types::ExitPoint::Address { address } => {
                proto::exit_node::ExitNodeEnum::Address(proto::Address {
                    nym_address: address.to_string(),
                    gateway_id: address.gateway().to_base58_string(),
                })
            }
            nym_vpn_lib_types::ExitPoint::Gateway { identity } => {
                proto::exit_node::ExitNodeEnum::Gateway(proto::GatewayId {
                    id: identity.to_base58_string(),
                })
            }
            nym_vpn_lib_types::ExitPoint::Country {
                two_letter_iso_country_code,
            } => proto::exit_node::ExitNodeEnum::Country(proto::Country {
                two_letter_iso_country_code,
            }),
            nym_vpn_lib_types::ExitPoint::Region { region } => {
                proto::exit_node::ExitNodeEnum::Region(proto::Region { region })
            }
            nym_vpn_lib_types::ExitPoint::Random => proto::exit_node::ExitNodeEnum::Random(()),
            nym_vpn_lib_types::ExitPoint::Auto {
                exclude_entry_point_country,
                exclude_user_country,
            } => proto::exit_node::ExitNodeEnum::Auto(proto::AutoExit {
                exclude_entry_point_country,
                exclude_user_country,
            }),
        };
        proto::ExitNode {
            exit_node_enum: Some(exit_node_enum),
        }
    }
}

impl From<nym_vpn_lib_types::GatewayFilter> for proto::GatewayFilter {
    fn from(value: nym_vpn_lib_types::GatewayFilter) -> Self {
        match value {
            nym_vpn_lib_types::GatewayFilter::MinScore(score) => proto::GatewayFilter {
                filter: Some(proto::gateway_filter::Filter::MinScore(score as i32)),
            },
            nym_vpn_lib_types::GatewayFilter::Country(country_code) => proto::GatewayFilter {
                filter: Some(proto::gateway_filter::Filter::Country(country_code)),
            },
            nym_vpn_lib_types::GatewayFilter::Region(region) => proto::GatewayFilter {
                filter: Some(proto::gateway_filter::Filter::Region(region)),
            },
            nym_vpn_lib_types::GatewayFilter::Residential => proto::GatewayFilter {
                filter: Some(proto::gateway_filter::Filter::Residential(())),
            },
            nym_vpn_lib_types::GatewayFilter::QuicEnabled => proto::GatewayFilter {
                filter: Some(proto::gateway_filter::Filter::QuicEnabled(())),
            },
            nym_vpn_lib_types::GatewayFilter::Exit => proto::GatewayFilter {
                filter: Some(proto::gateway_filter::Filter::Exit(())),
            },
            nym_vpn_lib_types::GatewayFilter::Vpn => proto::GatewayFilter {
                filter: Some(proto::gateway_filter::Filter::Vpn(())),
            },
        }
    }
}

impl TryFrom<proto::GatewayFilter> for nym_vpn_lib_types::GatewayFilter {
    type Error = ConversionError;

    fn try_from(value: proto::GatewayFilter) -> Result<Self, ConversionError> {
        Ok(
            match value
                .filter
                .ok_or_else(|| ConversionError::Generic("missing filter".to_string()))?
            {
                proto::gateway_filter::Filter::MinScore(score) => {
                    let score = proto::Score::try_from(score)
                        .map_err(|err| ConversionError::Decode("Score", err))?;
                    let score = nym_vpn_lib_types::Score::from(score);
                    nym_vpn_lib_types::GatewayFilter::MinScore(score)
                }
                proto::gateway_filter::Filter::Country(country_code) => {
                    nym_vpn_lib_types::GatewayFilter::Country(country_code)
                }
                proto::gateway_filter::Filter::Region(region) => {
                    nym_vpn_lib_types::GatewayFilter::Region(region)
                }
                proto::gateway_filter::Filter::Residential(()) => {
                    nym_vpn_lib_types::GatewayFilter::Residential
                }
                proto::gateway_filter::Filter::QuicEnabled(()) => {
                    nym_vpn_lib_types::GatewayFilter::QuicEnabled
                }
                proto::gateway_filter::Filter::Exit(()) => nym_vpn_lib_types::GatewayFilter::Exit,
                proto::gateway_filter::Filter::Vpn(()) => nym_vpn_lib_types::GatewayFilter::Vpn,
            },
        )
    }
}

impl From<nym_vpn_lib_types::LookupGatewayFilters> for proto::LookupGatewayFilters {
    fn from(value: nym_vpn_lib_types::LookupGatewayFilters) -> Self {
        Self {
            kind: value.gw_type as i32,
            filters: value.filters.into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<proto::LookupGatewayFilters> for nym_vpn_lib_types::LookupGatewayFilters {
    type Error = ConversionError;

    fn try_from(value: proto::LookupGatewayFilters) -> Result<Self, ConversionError> {
        let proto_gw_type = proto::GatewayType::try_from(value.kind)
            .map_err(|err| ConversionError::Decode("GatewayFilters.kind", err))?;
        let gw_type = nym_vpn_lib_types::GatewayType::from(proto_gw_type);

        let filters = value
            .filters
            .into_iter()
            .map(nym_vpn_lib_types::GatewayFilter::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { gw_type, filters })
    }
}

impl From<nym_vpn_lib_types::Score> for proto::Score {
    fn from(value: nym_vpn_lib_types::Score) -> Self {
        match value {
            nym_vpn_lib_types::Score::Offline => proto::Score::Offline,
            nym_vpn_lib_types::Score::Low => proto::Score::Low,
            nym_vpn_lib_types::Score::Medium => proto::Score::Medium,
            nym_vpn_lib_types::Score::High => proto::Score::High,
        }
    }
}

impl From<proto::Score> for nym_vpn_lib_types::Score {
    fn from(value: proto::Score) -> Self {
        match value {
            proto::Score::Offline => Self::Offline,
            proto::Score::Low => Self::Low,
            proto::Score::Medium => Self::Medium,
            proto::Score::High => Self::High,
        }
    }
}
