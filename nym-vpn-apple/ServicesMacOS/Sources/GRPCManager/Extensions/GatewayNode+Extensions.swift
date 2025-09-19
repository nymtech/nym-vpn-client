import CountriesManagerTypes

extension GatewayNode {
    init(with newGateway: NymVpnService_GatewayResponse) {
        self.init(
            id: newGateway.id.id,
            countryCode: newGateway.location.twoLetterIsoCountryCode,
            wgScore: GatewayNodeScore(with: newGateway.wgPerformance.score),
            mixnetScore: GatewayNodeScore(with: newGateway.mixnetScore),
            moniker: newGateway.moniker
        )
    }
}
