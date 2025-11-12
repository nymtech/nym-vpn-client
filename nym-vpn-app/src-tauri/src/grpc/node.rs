use nym_vpn_lib_types as lib;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
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
    Random,
}

impl TryFrom<Node> for lib::EntryPoint {
    type Error = anyhow::Error;

    fn try_from(node: Node) -> Result<Self, Self::Error> {
        Ok(match node {
            Node::Country(country) => country.into(),
            Node::Region(region) => region.into(),
            Node::Gateway(gateway) => gateway.try_into()?,
            Node::Random => lib::EntryPoint::Random,
        })
    }
}

impl TryFrom<Node> for lib::ExitPoint {
    type Error = anyhow::Error;

    fn try_from(node: Node) -> Result<Self, Self::Error> {
        Ok(match node {
            Node::Country(country) => country.into(),
            Node::Region(region) => region.into(),
            Node::Gateway(gateway) => gateway.try_into()?,
            Node::Random => lib::ExitPoint::Random,
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

impl From<RegionNode> for lib::EntryPoint {
    fn from(region: RegionNode) -> Self {
        lib::EntryPoint::Region {
            region: region.name,
        }
    }
}

impl From<RegionNode> for lib::ExitPoint {
    fn from(region: RegionNode) -> Self {
        lib::ExitPoint::Region {
            region: region.name,
        }
    }
}

impl TryFrom<GatewayNode> for lib::EntryPoint {
    type Error = anyhow::Error;

    fn try_from(gateway: GatewayNode) -> Result<Self, Self::Error> {
        let id = lib::NodeIdentity::from_str(&gateway.id);
        match id {
            Ok(identity) => Ok(lib::EntryPoint::Gateway { identity }),
            Err(err) => Err(anyhow::anyhow!(
                "failed to parse gateway id '{}': {}",
                gateway.id,
                err
            )),
        }
    }
}

impl TryFrom<GatewayNode> for lib::ExitPoint {
    type Error = anyhow::Error;

    fn try_from(gateway: GatewayNode) -> Result<Self, Self::Error> {
        let id = lib::NodeIdentity::from_str(&gateway.id);
        match id {
            Ok(identity) => Ok(lib::ExitPoint::Gateway { identity }),
            Err(err) => Err(anyhow::anyhow!(
                "failed to parse gateway id '{}': {}",
                gateway.id,
                err
            )),
        }
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Node::Country(country) => write!(f, "country {country}"),
            Node::Region(region) => write!(f, "region {region}"),
            Node::Gateway(gateway) => write!(f, "gateway {gateway}"),
            Node::Random => write!(f, "random"),
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
