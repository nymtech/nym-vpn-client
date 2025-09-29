import NymVPNRpc
import CountriesManagerTypes

extension GatewayNode {
    init(with newGateway: Gateway) {
        self.init(
            id: newGateway.identityKey,
            location: GatewayNodeLocation(with: newGateway.location),
            performance: GatewayNodePerformance(with: newGateway.wgPerformance),
            mixnetScore: GatewayNodeScore.convert(from: newGateway.mixnetScore) ?? .noScore,
            moniker: newGateway.moniker,
            buildVersion: newGateway.buildVersion,
            ipv4s: newGateway.exitIpv4S,
            ipv6s: newGateway.exitIpv6S
        )
    }
}
