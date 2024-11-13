use serde::Serialize;
use ts_rs::TS;

#[derive(Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AccountLinks {
    pub sign_up: Option<String>,
    pub sign_in: Option<String>,
    pub account: Option<String>,
}

impl From<nym_vpn_proto::AccountManagement> for AccountLinks {
    fn from(links: nym_vpn_proto::AccountManagement) -> Self {
        AccountLinks {
            sign_up: links.sign_up.map(|link| link.url.to_string()),
            sign_in: links.sign_in.map(|link| link.url.to_string()),
            account: links.account.map(|link| link.url.to_string()),
        }
    }
}
