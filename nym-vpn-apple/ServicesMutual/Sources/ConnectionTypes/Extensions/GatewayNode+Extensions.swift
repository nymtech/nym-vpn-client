import NymVPNLib

extension GatewayNode {
    public init(with gateway: Gateway) {
        self.init(
            id: gateway.identityKey,
            location: GatewayNodeLocation(with: gateway.location),
            performance: GatewayNodePerformance(with: gateway.performance),
            mixnetScore: GatewayNodeScore(with: gateway.performance?.mixnetScore),
            name: gateway.name,
            description: gateway.description,
            buildVersion: gateway.buildVersion,
            ipv4s: gateway.exitIpv4s,
            ipv6s: gateway.exitIpv6s,
            bridges: GatewayBridgeInformation(with: gateway.bridgeParams),
            operatorFamilyName: gateway.nodeFamilyName
        )
    }
}
