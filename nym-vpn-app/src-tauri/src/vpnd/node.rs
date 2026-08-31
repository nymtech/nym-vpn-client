use anyhow::anyhow;
use nym_vpn_lib_types as lib;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use ts_rs::TS;

use crate::country::Country;

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "tauri.ts")]
#[ts(rename = "SelectedNode")]
pub enum Node {
    Country {
        code: String,
    },
    Gateway {
        id: String,
    },
    Region(String),
    Random,
    Auto {
        exclude_user_country: bool,
        exclude_entry_point_country: bool,
    },
}

impl TryFrom<Node> for lib::EntryPoint {
    type Error = anyhow::Error;

    fn try_from(node: Node) -> Result<Self, Self::Error> {
        Ok(match node {
            Node::Country { code } => lib::EntryPoint::Country {
                two_letter_iso_country_code: code,
            },
            Node::Region(region) => lib::EntryPoint::Region { region },
            Node::Gateway { id } => lib::EntryPoint::Gateway {
                identity: lib::NodeIdentity::from_str(&id)?,
            },
            Node::Random => lib::EntryPoint::Random,
            Node::Auto {
                exclude_user_country,
                ..
            } => lib::EntryPoint::Auto {
                exclude_user_country,
            },
        })
    }
}

impl TryFrom<Node> for lib::ExitPoint {
    type Error = anyhow::Error;

    fn try_from(node: Node) -> Result<Self, Self::Error> {
        Ok(match node {
            Node::Country { code } => lib::ExitPoint::Country {
                two_letter_iso_country_code: code,
            },
            Node::Region(region) => lib::ExitPoint::Region { region },
            Node::Gateway { id } => lib::ExitPoint::Gateway {
                identity: lib::NodeIdentity::from_str(&id)?,
            },
            Node::Random => lib::ExitPoint::Random,
            Node::Auto {
                exclude_entry_point_country,
                exclude_user_country,
            } => lib::ExitPoint::Auto {
                exclude_entry_point_country,
                exclude_user_country,
            },
        })
    }
}

impl From<lib::EntryPoint> for Node {
    fn from(node: lib::EntryPoint) -> Self {
        match node {
            lib::EntryPoint::Country {
                two_letter_iso_country_code: code,
            } => Node::Country { code },
            lib::EntryPoint::Region { region } => Node::Region(region),
            lib::EntryPoint::Gateway { identity } => Node::Gateway {
                id: identity.to_base58_string(),
            },
            lib::EntryPoint::Random => Node::Random,
            lib::EntryPoint::Auto {
                exclude_user_country,
            } => Node::Auto {
                exclude_user_country,
                exclude_entry_point_country: exclude_user_country,
            },
        }
    }
}

impl TryFrom<lib::ExitPoint> for Node {
    type Error = anyhow::Error;

    fn try_from(node: lib::ExitPoint) -> Result<Self, Self::Error> {
        Ok(match node {
            lib::ExitPoint::Country {
                two_letter_iso_country_code: code,
            } => Node::Country { code },
            lib::ExitPoint::Region { region } => Node::Region(region),
            lib::ExitPoint::Gateway { identity } => Node::Gateway {
                id: identity.to_base58_string(),
            },
            lib::ExitPoint::Random => Node::Random,
            lib::ExitPoint::Address { address: _ } => {
                // TODO add support for this type of exit point
                return Err(anyhow!(
                    "Exit node of type [Address] is not supported by tauri client"
                ));
            }
            lib::ExitPoint::Auto {
                exclude_entry_point_country,
                exclude_user_country,
            } => Node::Auto {
                exclude_user_country,
                exclude_entry_point_country,
            },
        })
    }
}

impl From<Country> for lib::EntryPoint {
    fn from(country: Country) -> Self {
        lib::EntryPoint::Country {
            two_letter_iso_country_code: country.code,
        }
    }
}

impl From<Country> for lib::ExitPoint {
    fn from(country: Country) -> Self {
        lib::ExitPoint::Country {
            two_letter_iso_country_code: country.code,
        }
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Node::Country { code } => write!(f, "country [{}]", code.to_uppercase()),
            Node::Region(region) => write!(f, "region {region}"),
            Node::Gateway { id } => write!(f, "gateway [{id}]"),
            Node::Random => write!(f, "random"),
            Node::Auto {
                exclude_user_country,
                exclude_entry_point_country,
            } => write!(
                f,
                "auto exclude_user_country {exclude_user_country}, exclude_entry_point_country {exclude_entry_point_country}"
            ),
        }
    }
}
