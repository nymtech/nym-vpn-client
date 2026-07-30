// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub(crate) mod legacy {
    use crate::service::ConfigSetupError;
    use serde::Deserialize;

    #[derive(Clone, Debug, Deserialize)]
    pub(crate) enum EntryPoint {
        Gateway { identity: Vec<u8> },
        Location { location: String },
        Random,
    }

    impl TryFrom<EntryPoint> for nym_vpn_lib_types::EntryPoint {
        type Error = ConfigSetupError;

        fn try_from(value: EntryPoint) -> Result<Self, Self::Error> {
            match value {
                EntryPoint::Gateway { identity } => Ok(nym_vpn_lib_types::EntryPoint::Gateway {
                    identity: nym_vpn_lib_types::NodeIdentity::from_bytes(&identity)
                        .map_err(|e| ConfigSetupError::EntryPoint(e.to_string()))?,
                }),
                EntryPoint::Location { location } => Ok(nym_vpn_lib_types::EntryPoint::Country {
                    two_letter_iso_country_code: location,
                }),
                EntryPoint::Random => Ok(nym_vpn_lib_types::EntryPoint::Random),
            }
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
    pub(crate) enum ExitPoint {
        Address { address: String },
        Gateway { identity: Vec<u8> },
        Location { location: String },
        Random,
    }

    impl TryFrom<ExitPoint> for nym_vpn_lib_types::ExitPoint {
        type Error = ConfigSetupError;

        fn try_from(value: ExitPoint) -> Result<Self, Self::Error> {
            match value {
                ExitPoint::Address { address } => {
                    let recipient = nym_vpn_lib_types::Recipient::try_from_base58_string(&address)
                        .map_err(|e| ConfigSetupError::ExitPoint(e.to_string()))?;
                    Ok(nym_vpn_lib_types::ExitPoint::Address {
                        address: Box::new(recipient),
                    })
                }
                ExitPoint::Gateway { identity } => Ok(nym_vpn_lib_types::ExitPoint::Gateway {
                    identity: nym_vpn_lib_types::NodeIdentity::from_bytes(&identity)
                        .map_err(|e| ConfigSetupError::ExitPoint(e.to_string()))?,
                }),
                ExitPoint::Location { location } => Ok(nym_vpn_lib_types::ExitPoint::Country {
                    two_letter_iso_country_code: location,
                }),
                ExitPoint::Random => Ok(nym_vpn_lib_types::ExitPoint::Random),
            }
        }
    }
}

pub(crate) mod v1 {
    use crate::service::ConfigSetupError;
    use serde::{Deserialize, Serialize};
    use std::str::FromStr;

    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum EntryPoint {
        Gateway { identity: String },
        Location { location: String },
        Random,
    }

    impl TryFrom<EntryPoint> for nym_vpn_lib_types::EntryPoint {
        type Error = ConfigSetupError;

        fn try_from(value: EntryPoint) -> Result<Self, Self::Error> {
            match value {
                EntryPoint::Gateway { ref identity } => {
                    nym_vpn_lib_types::EntryPoint::from_base58_string(identity)
                        .map_err(|e| ConfigSetupError::EntryPoint(e.to_string()))
                }
                EntryPoint::Location { location } => Ok(nym_vpn_lib_types::EntryPoint::Country {
                    two_letter_iso_country_code: location,
                }),
                EntryPoint::Random => Ok(nym_vpn_lib_types::EntryPoint::Random),
            }
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
    #[serde(rename_all = "snake_case")]
    pub enum ExitPoint {
        Address { address: String },
        Gateway { identity: String },
        Location { location: String },
        Random,
    }

    impl TryFrom<ExitPoint> for nym_vpn_lib_types::ExitPoint {
        type Error = ConfigSetupError;

        fn try_from(value: ExitPoint) -> Result<Self, Self::Error> {
            match value {
                ExitPoint::Address { address } => {
                    let recipient = nym_vpn_lib_types::Recipient::try_from_base58_string(&address)
                        .map_err(|e| ConfigSetupError::ExitPoint(e.to_string()))?;
                    Ok(nym_vpn_lib_types::ExitPoint::Address {
                        address: Box::new(recipient),
                    })
                }
                ExitPoint::Gateway { identity } => {
                    let node_identity = nym_vpn_lib_types::NodeIdentity::from_str(&identity)
                        .map_err(|e| ConfigSetupError::ExitPoint(e.to_string()))?;
                    Ok(nym_vpn_lib_types::ExitPoint::Gateway {
                        identity: node_identity,
                    })
                }
                ExitPoint::Location { location } => Ok(nym_vpn_lib_types::ExitPoint::Country {
                    two_letter_iso_country_code: location,
                }),
                ExitPoint::Random => Ok(nym_vpn_lib_types::ExitPoint::Random),
            }
        }
    }
}

pub(crate) mod v2 {
    use crate::service::ConfigSetupError;
    use serde::{Deserialize, Serialize};
    use std::str::FromStr;

    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum EntryPoint {
        Gateway { identity: String },
        Country { two_letter_iso_country_code: String },
        Region { region: String },
        Random,
    }

    impl TryFrom<EntryPoint> for nym_vpn_lib_types::EntryPoint {
        type Error = ConfigSetupError;

        fn try_from(value: EntryPoint) -> Result<Self, Self::Error> {
            match value {
                EntryPoint::Gateway { ref identity } => {
                    nym_vpn_lib_types::EntryPoint::from_base58_string(identity)
                        .map_err(|e| ConfigSetupError::EntryPoint(e.to_string()))
                }
                EntryPoint::Country {
                    two_letter_iso_country_code,
                } => Ok(nym_vpn_lib_types::EntryPoint::Country {
                    two_letter_iso_country_code,
                }),
                EntryPoint::Region { region } => {
                    Ok(nym_vpn_lib_types::EntryPoint::Region { region })
                }
                EntryPoint::Random => Ok(nym_vpn_lib_types::EntryPoint::Random),
            }
        }
    }

    // This is only required for the latest version of the external entry point representation
    impl TryFrom<&nym_vpn_lib_types::EntryPoint> for EntryPoint {
        type Error = ConfigSetupError;

        fn try_from(value: &nym_vpn_lib_types::EntryPoint) -> Result<Self, Self::Error> {
            match value {
                nym_vpn_lib_types::EntryPoint::Gateway { identity } => Ok(EntryPoint::Gateway {
                    identity: identity.to_base58_string(),
                }),
                nym_vpn_lib_types::EntryPoint::Country {
                    two_letter_iso_country_code,
                } => Ok(EntryPoint::Country {
                    two_letter_iso_country_code: two_letter_iso_country_code.clone(),
                }),
                nym_vpn_lib_types::EntryPoint::Region { region } => Ok(EntryPoint::Region {
                    region: region.clone(),
                }),
                nym_vpn_lib_types::EntryPoint::Random => Ok(EntryPoint::Random),
                nym_vpn_lib_types::EntryPoint::Auto { .. } => Err(ConfigSetupError::EntryPoint(
                    String::from("no auto field in v2"),
                )),
            }
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
    #[serde(rename_all = "snake_case")]
    pub enum ExitPoint {
        Address { address: String },
        Gateway { identity: String },
        Country { two_letter_iso_country_code: String },
        Region { region: String },
        Random,
    }

    impl TryFrom<ExitPoint> for nym_vpn_lib_types::ExitPoint {
        type Error = ConfigSetupError;

        fn try_from(value: ExitPoint) -> Result<Self, Self::Error> {
            match value {
                ExitPoint::Address { address } => {
                    let recipient = nym_vpn_lib_types::Recipient::try_from_base58_string(&address)
                        .map_err(|e| ConfigSetupError::ExitPoint(e.to_string()))?;
                    Ok(nym_vpn_lib_types::ExitPoint::Address {
                        address: Box::new(recipient),
                    })
                }
                ExitPoint::Gateway { identity } => {
                    let node_identity = nym_vpn_lib_types::NodeIdentity::from_str(&identity)
                        .map_err(|e| ConfigSetupError::ExitPoint(e.to_string()))?;
                    Ok(nym_vpn_lib_types::ExitPoint::Gateway {
                        identity: node_identity,
                    })
                }
                ExitPoint::Country {
                    two_letter_iso_country_code,
                } => Ok(nym_vpn_lib_types::ExitPoint::Country {
                    two_letter_iso_country_code,
                }),
                ExitPoint::Region { region } => Ok(nym_vpn_lib_types::ExitPoint::Region { region }),
                ExitPoint::Random => Ok(nym_vpn_lib_types::ExitPoint::Random),
            }
        }
    }

    // This is only required for the latest version of the external exit point representation
    impl TryFrom<&nym_vpn_lib_types::ExitPoint> for ExitPoint {
        type Error = ConfigSetupError;

        fn try_from(value: &nym_vpn_lib_types::ExitPoint) -> Result<Self, Self::Error> {
            match value {
                nym_vpn_lib_types::ExitPoint::Address { address } => Ok(ExitPoint::Address {
                    address: address.to_string(),
                }),
                nym_vpn_lib_types::ExitPoint::Gateway { identity } => Ok(ExitPoint::Gateway {
                    identity: identity.to_string(),
                }),
                nym_vpn_lib_types::ExitPoint::Country {
                    two_letter_iso_country_code,
                } => Ok(ExitPoint::Country {
                    two_letter_iso_country_code: two_letter_iso_country_code.clone(),
                }),
                nym_vpn_lib_types::ExitPoint::Region { region } => Ok(ExitPoint::Region {
                    region: region.clone(),
                }),
                nym_vpn_lib_types::ExitPoint::Random => Ok(ExitPoint::Random),
                nym_vpn_lib_types::ExitPoint::Auto { .. } => Err(ConfigSetupError::ExitPoint(
                    String::from("no auto field in v2"),
                )),
            }
        }
    }
}

pub(crate) mod v3 {
    use crate::service::ConfigSetupError;
    use serde::{Deserialize, Serialize};
    use std::str::FromStr;

    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum EntryPoint {
        Gateway { identity: String },
        Country { two_letter_iso_country_code: String },
        Region { region: String },
        Random,
        Auto { exclude_user_country: bool },
    }

    impl TryFrom<EntryPoint> for nym_vpn_lib_types::EntryPoint {
        type Error = ConfigSetupError;

        fn try_from(value: EntryPoint) -> Result<Self, Self::Error> {
            match value {
                EntryPoint::Gateway { ref identity } => {
                    nym_vpn_lib_types::EntryPoint::from_base58_string(identity)
                        .map_err(|e| ConfigSetupError::EntryPoint(e.to_string()))
                }
                EntryPoint::Country {
                    two_letter_iso_country_code,
                } => Ok(nym_vpn_lib_types::EntryPoint::Country {
                    two_letter_iso_country_code,
                }),
                EntryPoint::Region { region } => {
                    Ok(nym_vpn_lib_types::EntryPoint::Region { region })
                }
                EntryPoint::Random => Ok(nym_vpn_lib_types::EntryPoint::Random),
                EntryPoint::Auto {
                    exclude_user_country,
                } => Ok(nym_vpn_lib_types::EntryPoint::Auto {
                    exclude_user_country,
                }),
            }
        }
    }

    // This is only required for the latest version of the external entry point representation
    impl TryFrom<&nym_vpn_lib_types::EntryPoint> for EntryPoint {
        type Error = ConfigSetupError;

        fn try_from(value: &nym_vpn_lib_types::EntryPoint) -> Result<Self, Self::Error> {
            match value {
                nym_vpn_lib_types::EntryPoint::Gateway { identity } => Ok(EntryPoint::Gateway {
                    identity: identity.to_base58_string(),
                }),
                nym_vpn_lib_types::EntryPoint::Country {
                    two_letter_iso_country_code,
                } => Ok(EntryPoint::Country {
                    two_letter_iso_country_code: two_letter_iso_country_code.clone(),
                }),
                nym_vpn_lib_types::EntryPoint::Region { region } => Ok(EntryPoint::Region {
                    region: region.clone(),
                }),
                nym_vpn_lib_types::EntryPoint::Random => Ok(EntryPoint::Random),
                nym_vpn_lib_types::EntryPoint::Auto {
                    exclude_user_country,
                } => Ok(EntryPoint::Auto {
                    exclude_user_country: *exclude_user_country,
                }),
            }
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
    #[serde(rename_all = "snake_case")]
    pub enum ExitPoint {
        Address {
            address: String,
        },
        Gateway {
            identity: String,
        },
        Country {
            two_letter_iso_country_code: String,
        },
        Region {
            region: String,
        },
        Random,
        Auto {
            exclude_entry_point_country: bool,
            exclude_user_country: bool,
        },
    }

    impl TryFrom<ExitPoint> for nym_vpn_lib_types::ExitPoint {
        type Error = ConfigSetupError;

        fn try_from(value: ExitPoint) -> Result<Self, Self::Error> {
            match value {
                ExitPoint::Address { address } => {
                    let recipient = nym_vpn_lib_types::Recipient::try_from_base58_string(&address)
                        .map_err(|e| ConfigSetupError::ExitPoint(e.to_string()))?;
                    Ok(nym_vpn_lib_types::ExitPoint::Address {
                        address: Box::new(recipient),
                    })
                }
                ExitPoint::Gateway { identity } => {
                    let node_identity = nym_vpn_lib_types::NodeIdentity::from_str(&identity)
                        .map_err(|e| ConfigSetupError::ExitPoint(e.to_string()))?;
                    Ok(nym_vpn_lib_types::ExitPoint::Gateway {
                        identity: node_identity,
                    })
                }
                ExitPoint::Country {
                    two_letter_iso_country_code,
                } => Ok(nym_vpn_lib_types::ExitPoint::Country {
                    two_letter_iso_country_code,
                }),
                ExitPoint::Region { region } => Ok(nym_vpn_lib_types::ExitPoint::Region { region }),
                ExitPoint::Random => Ok(nym_vpn_lib_types::ExitPoint::Random),
                ExitPoint::Auto {
                    exclude_entry_point_country,
                    exclude_user_country,
                } => Ok(nym_vpn_lib_types::ExitPoint::Auto {
                    exclude_entry_point_country,
                    exclude_user_country,
                }),
            }
        }
    }

    // This is only required for the latest version of the external exit point representation
    impl TryFrom<&nym_vpn_lib_types::ExitPoint> for ExitPoint {
        type Error = ConfigSetupError;

        fn try_from(value: &nym_vpn_lib_types::ExitPoint) -> Result<Self, Self::Error> {
            match value {
                nym_vpn_lib_types::ExitPoint::Address { address } => Ok(ExitPoint::Address {
                    address: address.to_string(),
                }),
                nym_vpn_lib_types::ExitPoint::Gateway { identity } => Ok(ExitPoint::Gateway {
                    identity: identity.to_string(),
                }),
                nym_vpn_lib_types::ExitPoint::Country {
                    two_letter_iso_country_code,
                } => Ok(ExitPoint::Country {
                    two_letter_iso_country_code: two_letter_iso_country_code.clone(),
                }),
                nym_vpn_lib_types::ExitPoint::Region { region } => Ok(ExitPoint::Region {
                    region: region.clone(),
                }),
                nym_vpn_lib_types::ExitPoint::Random => Ok(ExitPoint::Random),
                nym_vpn_lib_types::ExitPoint::Auto {
                    exclude_entry_point_country,
                    exclude_user_country,
                } => Ok(ExitPoint::Auto {
                    exclude_entry_point_country: *exclude_entry_point_country,
                    exclude_user_country: *exclude_user_country,
                }),
            }
        }
    }
}
