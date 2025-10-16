use nym_vpn_proto::proto as p;
use p::{EntryNode, ExitNode, GatewayId, entry_node::EntryNodeEnum, exit_node::ExitNodeEnum};
use serde::{Deserialize, Serialize};
use std::fmt;
use ts_rs::TS;

use super::gateway::Gateway;
use crate::country::Country;

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(rename_all = "lowercase")]
#[serde(untagged)]
#[ts(export, export_to = "tauri.ts")]
#[allow(clippy::large_enum_variant)]
pub enum NodeConnect {
    Country(Country),
    Gateway(Gateway),
}

impl From<Country> for EntryNode {
    fn from(country: Country) -> Self {
        EntryNode {
            entry_node_enum: Some(EntryNodeEnum::Country(p::Country {
                two_letter_iso_country_code: country.code,
            })),
        }
    }
}

impl From<Country> for ExitNode {
    fn from(country: Country) -> Self {
        ExitNode {
            exit_node_enum: Some(ExitNodeEnum::Country(p::Country {
                two_letter_iso_country_code: country.code,
            })),
        }
    }
}

impl From<Gateway> for EntryNode {
    fn from(gateway: Gateway) -> Self {
        EntryNode {
            entry_node_enum: Some(EntryNodeEnum::Gateway(GatewayId { id: gateway.id })),
        }
    }
}

impl From<Gateway> for ExitNode {
    fn from(gateway: Gateway) -> Self {
        ExitNode {
            exit_node_enum: Some(ExitNodeEnum::Gateway(GatewayId { id: gateway.id })),
        }
    }
}

impl From<NodeConnect> for EntryNode {
    fn from(node: NodeConnect) -> Self {
        match node {
            NodeConnect::Country(country) => country.into(),
            NodeConnect::Gateway(gateway) => gateway.into(),
        }
    }
}

impl From<NodeConnect> for ExitNode {
    fn from(node: NodeConnect) -> Self {
        match node {
            NodeConnect::Country(country) => country.into(),
            NodeConnect::Gateway(gateway) => gateway.into(),
        }
    }
}

impl fmt::Display for NodeConnect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeConnect::Country(country) => write!(f, "country {country}"),
            NodeConnect::Gateway(gateway) => write!(f, "gateway {gateway}"),
        }
    }
}
