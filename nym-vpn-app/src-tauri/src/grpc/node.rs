use crate::country::Country;
use nym_vpn_proto::entry_node::EntryNodeEnum;
use nym_vpn_proto::exit_node::ExitNodeEnum;
use nym_vpn_proto::{EntryNode, ExitNode, Location};

impl From<Country> for EntryNode {
    fn from(country: Country) -> Self {
        EntryNode {
            entry_node_enum: Some(EntryNodeEnum::Location(Location {
                two_letter_iso_country_code: country.code,
                latitude: None,
                longitude: None,
            })),
        }
    }
}

impl From<Country> for ExitNode {
    fn from(country: Country) -> Self {
        ExitNode {
            exit_node_enum: Some(ExitNodeEnum::Location(Location {
                two_letter_iso_country_code: country.code,
                latitude: None,
                longitude: None,
            })),
        }
    }
}
