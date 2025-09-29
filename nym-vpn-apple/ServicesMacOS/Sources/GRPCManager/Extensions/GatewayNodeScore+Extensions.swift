import NymVPNRpc
import CountriesManagerTypes

extension GatewayNodeScore {
    static func convert(from score: GatewayScore?) -> GatewayNodeScore? {
        guard let score else { return nil }
        switch score {
        case .high:
            return .high
        case .medium:
            return .medium
        case .low:
            return .low
        case .offline:
            return .offline
        }
    }
}
