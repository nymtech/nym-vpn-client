use crate::{conversions::ConversionError, proto};

impl From<nym_gateway_directory::GatewayType> for proto::GatewayType {
    fn from(value: nym_gateway_directory::GatewayType) -> Self {
        match value {
            nym_gateway_directory::GatewayType::MixnetEntry => proto::GatewayType::MixnetEntry,
            nym_gateway_directory::GatewayType::MixnetExit => proto::GatewayType::MixnetExit,
            nym_gateway_directory::GatewayType::Wg => proto::GatewayType::Wg,
        }
    }
}

impl From<proto::GatewayType> for nym_gateway_directory::GatewayType {
    fn from(value: proto::GatewayType) -> Self {
        match value {
            proto::GatewayType::MixnetEntry => nym_gateway_directory::GatewayType::MixnetEntry,
            proto::GatewayType::MixnetExit => nym_gateway_directory::GatewayType::MixnetExit,
            proto::GatewayType::Wg => nym_gateway_directory::GatewayType::Wg,
        }
    }
}

impl TryFrom<proto::EntryNode> for nym_gateway_directory::EntryPoint {
    type Error = ConversionError;

    fn try_from(value: proto::EntryNode) -> Result<Self, Self::Error> {
        let entry_enum_value = value
            .entry_node_enum
            .ok_or(ConversionError::NoValueSet("EntryNode.entry_node_enum"))?;

        Ok(match entry_enum_value {
            proto::entry_node::EntryNodeEnum::Gateway(gateway) => {
                let identity = nym_gateway_directory::NodeIdentity::from_base58_string(&gateway.id)
                    .map_err(|err| {
                        ConversionError::Generic(format!("failed to parse gateway id: {err}"))
                    })?;
                nym_gateway_directory::EntryPoint::Gateway { identity }
            }

            proto::entry_node::EntryNodeEnum::Country(country) => {
                nym_gateway_directory::EntryPoint::Country {
                    two_letter_iso_country_code: country.two_letter_iso_country_code.to_string(),
                }
            }
            proto::entry_node::EntryNodeEnum::Region(region) => {
                nym_gateway_directory::EntryPoint::Region {
                    region: region.region.to_string(),
                }
            }
            proto::entry_node::EntryNodeEnum::Random(_) => {
                nym_gateway_directory::EntryPoint::Random
            }
        })
    }
}

impl From<nym_gateway_directory::EntryPoint> for proto::EntryNode {
    fn from(value: nym_gateway_directory::EntryPoint) -> Self {
        match value {
            nym_gateway_directory::EntryPoint::Gateway { identity } => proto::EntryNode {
                entry_node_enum: Some(proto::entry_node::EntryNodeEnum::Gateway(
                    proto::GatewayId {
                        id: identity.to_base58_string(),
                    },
                )),
            },
            nym_gateway_directory::EntryPoint::Country {
                two_letter_iso_country_code,
            } => proto::EntryNode {
                entry_node_enum: Some(proto::entry_node::EntryNodeEnum::Country(proto::Country {
                    two_letter_iso_country_code,
                })),
            },
            nym_gateway_directory::EntryPoint::Region { region } => proto::EntryNode {
                entry_node_enum: Some(proto::entry_node::EntryNodeEnum::Region(proto::Region {
                    region,
                })),
            },
            nym_gateway_directory::EntryPoint::Random => proto::EntryNode {
                entry_node_enum: Some(proto::entry_node::EntryNodeEnum::Random(())),
            },
        }
    }
}

impl TryFrom<proto::ExitNode> for nym_gateway_directory::ExitPoint {
    type Error = ConversionError;

    fn try_from(value: proto::ExitNode) -> Result<Self, Self::Error> {
        let exit_enum_value = value
            .exit_node_enum
            .ok_or(ConversionError::NoValueSet("ExitNode.exit_node_enum"))?;

        Ok(match exit_enum_value {
            proto::exit_node::ExitNodeEnum::Address(address) => {
                let address = nym_gateway_directory::Recipient::try_from_base58_string(
                    address.nym_address.clone(),
                )
                .map_err(|err| {
                    ConversionError::Generic(format!("failed to parse exit node address: {err}"))
                })?;
                nym_gateway_directory::ExitPoint::Address {
                    address: Box::new(address),
                }
            }
            proto::exit_node::ExitNodeEnum::Gateway(gateway) => {
                let identity = nym_gateway_directory::NodeIdentity::from_base58_string(&gateway.id)
                    .map_err(|err| {
                        ConversionError::Generic(format!("failed to parse gateway id: {err}"))
                    })?;
                nym_gateway_directory::ExitPoint::Gateway { identity }
            }
            proto::exit_node::ExitNodeEnum::Country(country) => {
                nym_gateway_directory::ExitPoint::Country {
                    two_letter_iso_country_code: country.two_letter_iso_country_code.to_string(),
                }
            }
            proto::exit_node::ExitNodeEnum::Region(region) => {
                nym_gateway_directory::ExitPoint::Region {
                    region: region.region.to_string(),
                }
            }
            proto::exit_node::ExitNodeEnum::Random(_) => nym_gateway_directory::ExitPoint::Random,
        })
    }
}

impl From<nym_gateway_directory::ExitPoint> for proto::ExitNode {
    fn from(value: nym_gateway_directory::ExitPoint) -> Self {
        let exit_node_enum = match value {
            nym_gateway_directory::ExitPoint::Address { address } => {
                proto::exit_node::ExitNodeEnum::Address(proto::Address {
                    nym_address: address.to_string(),
                    gateway_id: address.gateway().to_base58_string(),
                })
            }
            nym_gateway_directory::ExitPoint::Gateway { identity } => {
                proto::exit_node::ExitNodeEnum::Gateway(proto::GatewayId {
                    id: identity.to_base58_string(),
                })
            }
            nym_gateway_directory::ExitPoint::Country {
                two_letter_iso_country_code,
            } => proto::exit_node::ExitNodeEnum::Country(proto::Country {
                two_letter_iso_country_code,
            }),
            nym_gateway_directory::ExitPoint::Region { region } => {
                proto::exit_node::ExitNodeEnum::Region(proto::Region { region })
            }
            nym_gateway_directory::ExitPoint::Random => proto::exit_node::ExitNodeEnum::Random(()),
        };
        proto::ExitNode {
            exit_node_enum: Some(exit_node_enum),
        }
    }
}

impl From<nym_gateway_directory::GatewayFilter> for proto::GatewayFilter {
    fn from(value: nym_gateway_directory::GatewayFilter) -> Self {
        match value {
            nym_gateway_directory::GatewayFilter::MinScore(score) => proto::GatewayFilter {
                filter: Some(proto::gateway_filter::Filter::MinScore(score as i32)),
            },
            nym_gateway_directory::GatewayFilter::Country(country_code) => proto::GatewayFilter {
                filter: Some(proto::gateway_filter::Filter::Country(country_code)),
            },
            nym_gateway_directory::GatewayFilter::Region(region) => proto::GatewayFilter {
                filter: Some(proto::gateway_filter::Filter::Region(region)),
            },
            nym_gateway_directory::GatewayFilter::Residential => proto::GatewayFilter {
                filter: Some(proto::gateway_filter::Filter::Residential(())),
            },
            nym_gateway_directory::GatewayFilter::Exit => proto::GatewayFilter {
                filter: Some(proto::gateway_filter::Filter::Exit(())),
            },
            nym_gateway_directory::GatewayFilter::Vpn => proto::GatewayFilter {
                filter: Some(proto::gateway_filter::Filter::Vpn(())),
            },
        }
    }
}

impl TryFrom<proto::GatewayFilter> for nym_gateway_directory::GatewayFilter {
    type Error = ConversionError;

    fn try_from(value: proto::GatewayFilter) -> Result<Self, ConversionError> {
        Ok(
            match value
                .filter
                .ok_or_else(|| ConversionError::Generic("missing filter".to_string()))?
            {
                proto::gateway_filter::Filter::MinScore(score) => {
                    nym_gateway_directory::GatewayFilter::MinScore(
                        nym_gateway_directory::ScoreValue::try_from(
                            proto::Score::try_from(score)
                                .map_err(|err| ConversionError::Decode("Score", err))?,
                        )?,
                    )
                }
                proto::gateway_filter::Filter::Country(country_code) => {
                    nym_gateway_directory::GatewayFilter::Country(country_code)
                }
                proto::gateway_filter::Filter::Region(region) => {
                    nym_gateway_directory::GatewayFilter::Region(region)
                }
                proto::gateway_filter::Filter::Residential(()) => {
                    nym_gateway_directory::GatewayFilter::Residential
                }
                proto::gateway_filter::Filter::Exit(()) => {
                    nym_gateway_directory::GatewayFilter::Exit
                }
                proto::gateway_filter::Filter::Vpn(()) => nym_gateway_directory::GatewayFilter::Vpn,
            },
        )
    }
}

impl From<nym_gateway_directory::GatewayFilters> for proto::GatewayFilters {
    fn from(value: nym_gateway_directory::GatewayFilters) -> Self {
        Self {
            kind: value.gw_type as i32,
            filters: value.filters.into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<proto::GatewayFilters> for nym_gateway_directory::GatewayFilters {
    type Error = ConversionError;

    fn try_from(value: proto::GatewayFilters) -> Result<Self, ConversionError> {
        let proto_gw_type = proto::GatewayType::try_from(value.kind)
            .map_err(|err| ConversionError::Decode("GatewayFilters.kind", err))?;
        let gw_type = nym_gateway_directory::GatewayType::from(proto_gw_type);

        let filters = value
            .filters
            .into_iter()
            .map(nym_gateway_directory::GatewayFilter::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { gw_type, filters })
    }
}

impl From<nym_gateway_directory::ScoreValue> for proto::Score {
    fn from(value: nym_gateway_directory::ScoreValue) -> Self {
        match value {
            nym_gateway_directory::ScoreValue::Offline => proto::Score::Offline,
            nym_gateway_directory::ScoreValue::Low => proto::Score::Low,
            nym_gateway_directory::ScoreValue::Medium => proto::Score::Medium,
            nym_gateway_directory::ScoreValue::High => proto::Score::High,
        }
    }
}

impl TryFrom<proto::Score> for nym_gateway_directory::ScoreValue {
    type Error = ConversionError;

    fn try_from(value: proto::Score) -> Result<Self, ConversionError> {
        match value {
            proto::Score::Offline => Ok(nym_gateway_directory::ScoreValue::Offline),
            proto::Score::Low => Ok(nym_gateway_directory::ScoreValue::Low),
            proto::Score::Medium => Ok(nym_gateway_directory::ScoreValue::Medium),
            proto::Score::High => Ok(nym_gateway_directory::ScoreValue::High),
        }
    }
}
