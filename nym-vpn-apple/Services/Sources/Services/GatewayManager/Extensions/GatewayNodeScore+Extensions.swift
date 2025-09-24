#if os(iOS)
import CountriesManagerTypes
import NymVPNLib

extension GatewayNodeScore {
    init(with score: Score?) {
        guard let score
        else {
            self = .noScore
            return
        }
        switch score {
        case .high:
            self = .high
        case .medium:
            self = .medium
        case .low:
            self = .low
        case .offline:
            self = .offline
        }
    }
}
#endif
