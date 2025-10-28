use nym_vpn_proto::proto as p;
use p::{EntryNode, ExitNode, GatewayId, entry_node::EntryNodeEnum, exit_node::ExitNodeEnum};
use serde::{Deserialize, Serialize};
use std::fmt;
use ts_rs::TS;

use crate::country::Country;

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "tauri.ts")]
pub struct RegionNode {
    name: String,
    country: Country,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "tauri.ts")]
pub struct GatewayNode {
    pub id: String,
    pub name: String,
    pub country: Country,
    pub region: String,
    pub city: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "tauri.ts")]
#[ts(rename = "SelectedNode")]
#[serde(tag = "type", content = "node")]
pub enum Node {
    Country(Country),
    Gateway(GatewayNode),
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

impl From<GatewayNode> for EntryNode {
    fn from(gateway: GatewayNode) -> Self {
        EntryNode {
            entry_node_enum: Some(EntryNodeEnum::Gateway(GatewayId { id: gateway.id })),
        }
    }
}

impl From<GatewayNode> for ExitNode {
    fn from(gateway: GatewayNode) -> Self {
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
        write!(
            f,
            "region: [{}] {}, {}",
            self.country.code, self.country.name, self.name
        )
    }
}

impl fmt::Display for GatewayNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "gateway: [{}] {}, {}", self.id, self.name, self.country)
    }
}
