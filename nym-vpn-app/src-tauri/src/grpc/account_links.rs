use serde::Serialize;
use ts_rs::TS;

#[derive(Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, export_to = "tauri.ts")]
pub struct AccountLinks {
    pub sign_up: String,
    pub sign_in: String,
    pub account: Option<String>,
}

impl From<nym_vpn_proto::proto::AccountManagement> for AccountLinks {
    fn from(links: nym_vpn_proto::proto::AccountManagement) -> Self {
        AccountLinks {
            sign_up: links.sign_up,
            sign_in: links.sign_in,
            account: links.account,
        }
    }
}
