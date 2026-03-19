use nym_vpn_lib_types as lib;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, TS)]
#[ts(export, export_to = "tauri.ts", rename = "DeeplinkKind")]
#[serde(rename_all = "camelCase")]
pub enum DeeplinkKind {
    Privy,
    PrivyLink,
    AutologinRenew,
    AutologinView,
    CreateAccount,
}

impl From<lib::DeeplinkKind> for DeeplinkKind {
    fn from(kind: lib::DeeplinkKind) -> Self {
        match kind {
            lib::DeeplinkKind::Privy => DeeplinkKind::Privy,
            lib::DeeplinkKind::PrivyLink => DeeplinkKind::PrivyLink,
            lib::DeeplinkKind::AutologinRenew => DeeplinkKind::AutologinRenew,
            lib::DeeplinkKind::AutologinView => DeeplinkKind::AutologinView,
            lib::DeeplinkKind::CreateAccount => DeeplinkKind::CreateAccount,
        }
    }
}

impl From<DeeplinkKind> for lib::DeeplinkKind {
    fn from(kind: DeeplinkKind) -> Self {
        match kind {
            DeeplinkKind::Privy => lib::DeeplinkKind::Privy,
            DeeplinkKind::PrivyLink => lib::DeeplinkKind::PrivyLink,
            DeeplinkKind::AutologinRenew => lib::DeeplinkKind::AutologinRenew,
            DeeplinkKind::AutologinView => lib::DeeplinkKind::AutologinView,
            DeeplinkKind::CreateAccount => lib::DeeplinkKind::CreateAccount,
        }
    }
}
