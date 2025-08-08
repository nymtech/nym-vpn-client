use nym_vpn_proto::proto::{AccountControllerState, TunnelEvent};

#[allow(clippy::large_enum_variant)]
pub enum VpndEvent {
    Tunnel(TunnelEvent),
    Account(AccountControllerState),
}

impl From<TunnelEvent> for VpndEvent {
    fn from(event: TunnelEvent) -> Self {
        VpndEvent::Tunnel(event)
    }
}

impl From<AccountControllerState> for VpndEvent {
    fn from(event: AccountControllerState) -> Self {
        VpndEvent::Account(event)
    }
}
