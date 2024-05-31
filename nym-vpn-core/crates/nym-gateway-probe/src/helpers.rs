use nym_client_core::HardcodedTopologyProvider;
use nym_gateway_directory::{
    Config as GatewayDirectoryConfig, DescribedGatewayWithLocation,
    GatewayClient as GatewayDirectoryClient,
};
use nym_sdk::mixnet::{MixnetClientBuilder, NymTopology};

pub(crate) trait MaybeInjectableGateway: Sized {
    async fn maybe_inject_gateway(
        self,
        hardcoded: Option<DescribedGatewayWithLocation>,
    ) -> anyhow::Result<Self>;
}

impl MaybeInjectableGateway for MixnetClientBuilder {
    async fn maybe_inject_gateway(
        self,
        hardcoded: Option<DescribedGatewayWithLocation>,
    ) -> anyhow::Result<Self> {
        let Some(gateway) = hardcoded else {
            return Ok(self);
        };

        // that's a nasty hack, but will work for the one-off probe.
        let gateway_config = GatewayDirectoryConfig::new_from_env();
        let gateway_client = GatewayDirectoryClient::new(gateway_config.clone())?;
        let mixnodes = gateway_client.lookup_current_mixnodes().await?;
        let topology = NymTopology::from_detailed(mixnodes, vec![gateway.gateway.bond]);

        unimplemented!("we also require injecting this topology into `connect_to_mixnet` for initial gateway selection");

        Ok(self.custom_topology_provider(Box::new(HardcodedTopologyProvider::new(topology))))
    }
}
