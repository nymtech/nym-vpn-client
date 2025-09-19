#if os(iOS)
import CountriesManagerTypes
import NymVPNLib

extension GatewayNode {
    init(with gatewayInfo: GatewayInfo) {
        self.init(
            id: gatewayInfo.id,
            countryCode: gatewayInfo.location?.twoLetterIsoCountryCode ?? "",
            wgScore: GatewayNodeScore(with: gatewayInfo.wgPerformance?.score),
            mixnetScore: GatewayNodeScore(with: gatewayInfo.mixnetScore ?? .none),
            moniker: gatewayInfo.moniker
        )
    }
}
#endif
