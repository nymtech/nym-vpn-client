use std::net::IpAddr;

use nym_gateway_directory::{GatewayClient, IpPacketRouterAddress, Recipient};
use nym_ip_packet_client::IprClientConnect;
use nym_ip_packet_requests::IpPair;
use nym_task::TaskManager;

use super::connected_tunnel::ConnectedTunnel;
use crate::{
    mixnet::SharedMixnetClient,
    tunnel_state_machine::tunnel::{gateway_selector::SelectedGateways, Error, Result},
};

/// Struct holding addresses assigned by mixnet upon connect.
pub struct AssignedAddresses {
    pub entry_mixnet_gateway_ip: IpAddr,
    pub mixnet_client_address: Recipient,
    pub exit_mix_addresses: IpPacketRouterAddress,
    pub interface_addresses: IpPair,
}

/// Type responsible for connecting the mixnet tunnel.
pub struct Connector {
    task_manager: TaskManager,
    mixnet_client: SharedMixnetClient,
    gateway_directory_client: GatewayClient,
}

impl Connector {
    pub fn new(
        task_manager: TaskManager,
        mixnet_client: SharedMixnetClient,
        gateway_directory_client: GatewayClient,
    ) -> Self {
        Self {
            task_manager,
            mixnet_client,
            gateway_directory_client,
        }
    }

    pub async fn connect(
        self,
        selected_gateways: SelectedGateways,
        nym_ips: Option<IpPair>,
    ) -> Result<ConnectedTunnel> {
        let mixnet_client_address = self.mixnet_client.nym_address().await;
        let gateway_used = mixnet_client_address.gateway().to_base58_string();
        let entry_mixnet_gateway_ip: IpAddr = self
            .gateway_directory_client
            .lookup_gateway_ip(&gateway_used)
            .await
            .map_err(|source| Error::LookupGatewayIp {
                gateway_id: gateway_used,
                source,
            })?;

        let exit_mix_addresses = selected_gateways.exit.ipr_address.unwrap();

        let mut ipr_client = IprClientConnect::new_from_inner(self.mixnet_client.inner()).await;
        let interface_addresses = ipr_client
            .connect(exit_mix_addresses.0, nym_ips)
            .await
            .map_err(Error::ConnectToIpPacketRouter)?;

        let assigned_addresses = AssignedAddresses {
            entry_mixnet_gateway_ip,
            mixnet_client_address,
            exit_mix_addresses,
            interface_addresses,
        };

        Ok(ConnectedTunnel::new(
            self.task_manager,
            self.mixnet_client,
            assigned_addresses,
        ))
    }
}
