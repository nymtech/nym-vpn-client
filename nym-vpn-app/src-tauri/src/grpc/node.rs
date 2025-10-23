use nym_vpn_proto::proto as p;
use p::{EntryNode, ExitNode, GatewayId, entry_node::EntryNodeEnum, exit_node::ExitNodeEnum};
use serde::{Deserialize, Serialize};
use std::fmt;
use ts_rs::TS;

use super::gateway::Gateway;
use crate::country::Country;

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "tauri.ts")]
pub struct RegionNode {
    name: String,
    country: Country,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(rename_all = "lowercase")]
#[serde(untagged)]
#[ts(export, export_to = "tauri.ts")]
#[ts(rename = "SelectedNode")]
#[allow(clippy::large_enum_variant)]
pub enum Node {
    Country(Country),
    Gateway(Gateway),
    Region(RegionNode),
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

impl From<RegionNode> for EntryNode {
    fn from(region: RegionNode) -> Self {
        EntryNode {
            entry_node_enum: Some(EntryNodeEnum::Region(p::Region {
                region: region.name,
            })),
        }
    }
}

impl From<RegionNode> for ExitNode {
    fn from(region: RegionNode) -> Self {
        ExitNode {
            exit_node_enum: Some(ExitNodeEnum::Region(p::Region {
                region: region.name,
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

impl From<Node> for EntryNode {
    fn from(node: Node) -> Self {
        match node {
            Node::Country(country) => country.into(),
            Node::Region(region) => region.into(),
            Node::Gateway(gateway) => gateway.into(),
        }
    }
}

impl From<Node> for ExitNode {
    fn from(node: Node) -> Self {
        match node {
            Node::Country(country) => country.into(),
            Node::Region(region) => region.into(),
            Node::Gateway(gateway) => gateway.into(),
        }
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Node::Country(country) => write!(f, "country {country}"),
            Node::Region(region) => write!(f, "region {region}"),
            Node::Gateway(gateway) => write!(f, "gateway {gateway}"),
        }
    }
}

impl fmt::Display for RegionNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.country.code, self.name)
    }
}
