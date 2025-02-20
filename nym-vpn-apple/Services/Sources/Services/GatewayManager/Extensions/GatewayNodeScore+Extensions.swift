#if os(iOS)
import CountriesManagerTypes
import MixnetLibrary

extension GatewayNodeScore {
    init(with score: Score) {
        switch score {
        case .high:
            self = .high
        case .medium:
            self = .medium
        case .low:
            self = .low
        case .none:
            self = .noScore
        }
    }
}
#endif
