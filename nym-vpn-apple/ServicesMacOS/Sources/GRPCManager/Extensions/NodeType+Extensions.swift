import CountriesManagerTypes

extension NodeType {
    func convertToGatewayType() -> Nym_Vpn_GatewayType {
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
