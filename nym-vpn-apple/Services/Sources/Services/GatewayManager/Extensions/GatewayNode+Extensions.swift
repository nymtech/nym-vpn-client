#if os(iOS)
import CountriesManagerTypes
import NymVPNLib

extension GatewayNode {
    init(with gatewayInfo: Gateway) {
        self.init(
            id: gatewayInfo.identityKey,
            location: GatewayNodeLocation(with: gatewayInfo.location),
            performance: GatewayNodePerformance(with: gatewayInfo.performance),
            mixnetScore: GatewayNodeScore(with: gatewayInfo.performance?.mixnetScore ?? .none),
            name: gatewayInfo.name,
            description: gatewayInfo.description,
            buildVersion: gatewayInfo.buildVersion,
            ipv4s: gatewayInfo.exitIpv4s,
            ipv6s: gatewayInfo.exitIpv6s
        )
    }
}
#endif
