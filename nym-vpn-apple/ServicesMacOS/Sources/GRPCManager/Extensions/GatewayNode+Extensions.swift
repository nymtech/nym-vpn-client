import CountriesManagerTypes

extension GatewayNode {
    init(with newGateway: NymVpnService_GatewayResponse) {
        self.init(
            id: newGateway.id.id,
            countryCode: newGateway.location.twoLetterIsoCountryCode,
            city: newGateway.location.city,
            region: newGateway.location.region,
            asn: GatewayASN(with: newGateway.location.asn),
            performance: GatewayPerformance(with: newGateway.wgPerformance),
            mixnetScore: GatewayNodeScore(with: newGateway.mixnetScore),
            moniker: newGateway.moniker,
            buildVersion: newGateway.buildVersion,
            ipv4s: newGateway.exitIpv4S,
            ipv6s: newGateway.exitIpv6S
        )
    }
}
