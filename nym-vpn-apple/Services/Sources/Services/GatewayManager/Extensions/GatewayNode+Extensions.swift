#if os(iOS)
import CountriesManagerTypes
import NymVPNLib

extension GatewayNode {
    init(with gatewayInfo: GatewayInfo) {
        self.init(
            id: gatewayInfo.id,
            countryCode: gatewayInfo.location?.twoLetterIsoCountryCode ?? "",
            city: gatewayInfo.location?.city ?? "",
            region: gatewayInfo.location?.region ?? "",
            asn: GatewayASN(with: gatewayInfo.location?.asn),
            performance: GatewayPerformance(with: gatewayInfo.wgPerformance),
            mixnetScore: GatewayNodeScore(with: gatewayInfo.mixnetScore ?? .none),
            moniker: gatewayInfo.moniker,
            buildVersion: gatewayInfo.buildVersion,
            ipv4s: gatewayInfo.exitIpv4s,
            ipv6s: gatewayInfo.exitIpv6s
        )
    }
}
#endif
