use nym_connection_monitor::ConnectionStatusEvent;

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct PingResult {
    pub entry_gateway: String,
    pub exit_gateway: String,
    pub outcome: PingOutcome,
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum PingOutcome {
    EntryGatewayNotConnected,
    EntryGatewayNotRouting,
    ExitGatewayNotConnected,
    IpPingReplies(IpPingReplies),
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct IpPingReplies {
    pub ipr_tun_ip_v4: bool,
    pub ipr_tun_ip_v6: bool,
    pub external_ip_v4: bool,
    pub external_ip_v6: bool,
}

impl IpPingReplies {
    pub fn new() -> Self {
        Self {
            ipr_tun_ip_v4: false,
            ipr_tun_ip_v6: false,
            external_ip_v4: false,
            external_ip_v6: false,
        }
    }

    pub fn register_event(&mut self, event: &ConnectionStatusEvent) {
        match event {
            ConnectionStatusEvent::MixnetSelfPing => {}
            ConnectionStatusEvent::Icmpv4IprTunDevicePingReply => self.ipr_tun_ip_v4 = true,
            ConnectionStatusEvent::Icmpv6IprTunDevicePingReply => self.ipr_tun_ip_v6 = true,
            ConnectionStatusEvent::Icmpv4IprExternalPingReply => self.external_ip_v4 = true,
            ConnectionStatusEvent::Icmpv6IprExternalPingReply => self.external_ip_v6 = true,
        }
    }
}
