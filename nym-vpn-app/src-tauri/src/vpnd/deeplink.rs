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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_from_lib_kind() {
        assert_eq!(
            DeeplinkKind::from(lib::DeeplinkKind::CreateAccount),
            DeeplinkKind::CreateAccount
        );
    }

    #[test]
    fn round_trips_every_variant_through_the_lib_type() {
        let variants = [
            DeeplinkKind::Privy,
            DeeplinkKind::PrivyLink,
            DeeplinkKind::AutologinRenew,
            DeeplinkKind::AutologinView,
            DeeplinkKind::CreateAccount,
        ];
        for kind in variants {
            let lib_kind: lib::DeeplinkKind = kind.clone().into();
            let back: DeeplinkKind = lib_kind.into();
            assert_eq!(kind, back);
        }
    }
}
