import NymVPNRpc
import CountriesManagerTypes

extension NodeType {
    func convertToGatewayType() -> GatewayType {
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
