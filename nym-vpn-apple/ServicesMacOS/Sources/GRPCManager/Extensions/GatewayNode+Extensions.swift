import NymVPNRpc
import CountriesManagerTypes

extension GatewayNode {
    init(with newGateway: Gateway) {
        self.init(
            id: newGateway.identityKey,
            location: GatewayNodeLocation(with: newGateway.location),
            performance: GatewayNodePerformance(with: newGateway.performance),
            mixnetScore: GatewayNodeScore.convert(from: newGateway.performance?.mixnetScore) ?? .noScore,
            name: newGateway.name,
            description: newGateway.description,
            buildVersion: newGateway.buildVersion,
            ipv4s: newGateway.exitIpv4s,
            ipv6s: newGateway.exitIpv6s,
            bridges: GatewayBridgeInformation(with: newGateway.bridgeParams)
        )
    }
}
