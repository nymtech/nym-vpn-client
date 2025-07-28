import CountriesManagerTypes

extension NodeType {
    func convertToGatewayType() -> NymVpnService_GatewayType {
        switch self {
        case .entry:
            .mixnetEntry
        case .exit:
            .mixnetExit
        case .vpn:
            .wg
        }
    }
}
