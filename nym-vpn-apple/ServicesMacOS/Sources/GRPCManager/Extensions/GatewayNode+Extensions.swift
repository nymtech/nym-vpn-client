import CountriesManagerTypes

extension GatewayNode {
    init(with newGateway: Nym_Vpn_GatewayResponse) {
        self.init(
            id: newGateway.id.id,
            countryCode: newGateway.location.twoLetterIsoCountryCode,
            wgScore: GatewayNodeScore(with: newGateway.wgScore),
            mixnetScore: GatewayNodeScore(with: newGateway.mixnetScore),
            moniker: newGateway.moniker
        )
    }
}
