import CountriesManagerTypes
import NymVPNRpc

extension GatewayNodeASNType {
    init(with type: GatewayAsnKind) {
        switch type {
        case .residential:
            self = .residential
        case .other:
            self = .other
        }
    }
}
