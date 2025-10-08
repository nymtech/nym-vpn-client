#if os(iOS)
import CountriesManagerTypes
import NymVPNLib

extension GatewayNode {
    init(with gatewayInfo: Gateway) {
        self.init(
            id: gatewayInfo.identityKey,
            location: GatewayNodeLocation(with: gatewayInfo.location),
            performance: GatewayNodePerformance(with: gatewayInfo.wgPerformance),
            mixnetScore: GatewayNodeScore(with: gatewayInfo.mixnetScore ?? .none),
            moniker: gatewayInfo.moniker,
            buildVersion: gatewayInfo.buildVersion,
            ipv4s: gatewayInfo.exitIpv4s,
            ipv6s: gatewayInfo.exitIpv6s
        )
    }
}
#endif
