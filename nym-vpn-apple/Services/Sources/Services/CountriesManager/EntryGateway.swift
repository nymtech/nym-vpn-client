public enum EntryGateway: Codable, Equatable {
    case country(code: String)
    case lowLatencyCountry(code: String)
    case randomLowLatency
    case random

    public var countryCode: String? {
        switch self {
        case let .country(code: countryCode), let .lowLatencyCountry(code: countryCode):
            return countryCode
        case .randomLowLatency, .random:
            return nil
        }
    }

    public var isQuickest: Bool {
        switch self {
        case .country, .random:
            return false
        case .randomLowLatency, .lowLatencyCountry:
            return true
        }
    }

    public var isCountry: Bool {
        switch self {
        case .country:
            return true
        case .lowLatencyCountry, .randomLowLatency, .random:
            return false
        }
    }
}
