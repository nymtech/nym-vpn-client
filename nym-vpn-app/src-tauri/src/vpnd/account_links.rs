use nym_vpn_lib_types as lib;
use serde::Serialize;
use ts_rs::TS;

#[derive(Clone, Serialize, TS, Debug)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "tauri.ts")]
pub struct AccountLinks {
    pub sign_up: String,
    pub sign_in: String,
    pub account: Option<String>,
}

impl From<lib::ParsedAccountLinks> for AccountLinks {
    fn from(links: lib::ParsedAccountLinks) -> Self {
        AccountLinks {
            sign_up: links.sign_up,
            sign_in: links.sign_in,
            account: links.account,
        }
    }
}
