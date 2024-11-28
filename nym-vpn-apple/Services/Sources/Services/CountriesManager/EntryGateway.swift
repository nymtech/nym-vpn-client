public enum EntryGateway: Codable, Equatable {
    case country(code: String)
    case lowLatencyCountry(code: String)
    case gateway(String)
    case randomLowLatency
    case random

    public var countryCode: String? {
        switch self {
        case let .country(code: countryCode), let .lowLatencyCountry(code: countryCode):
            countryCode
        case .randomLowLatency, .random:
            nil
        case .gateway:
            nil
        }
    }

    public var isQuickest: Bool {
        switch self {
        case .country, .random:
            false
        case .randomLowLatency, .lowLatencyCountry, .gateway:
            true
        }
    }

    public var isCountry: Bool {
        switch self {
        case .country:
            true
        case .lowLatencyCountry, .randomLowLatency, .random, .gateway:
            false
        }
    }
}
