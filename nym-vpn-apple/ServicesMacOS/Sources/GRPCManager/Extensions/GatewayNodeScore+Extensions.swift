import NymVPNRpc
import CountriesManagerTypes

extension GatewayNodeScore {
    static func convert(from score: GatewayScore?) -> GatewayNodeScore? {
        switch score {
        case .high:
            .high
        case .medium:
            .medium
        case .low:
            .low
        case .none:
            .noScore
        case .none?:
            .noScore
        }
    }
}
