public enum ExitRouter: Codable, Equatable {
    case country(code: String)
    // Fictional, just country under the hood, while we get the actual functionality implemented
    case lowLatencyCountry(code: String)
    // Fictional, while we get the actual functionality implemented
    case randomLowLatency

    public var countryCode: String? {
        switch self {
        case let .country(code: countryCode):
            return countryCode
        case let .lowLatencyCountry(code: countryCode):
            return countryCode
        case .randomLowLatency:
            return nil
        }
    }

    public var isQuickest: Bool {
        switch self {
        case .country:
            return false
        case .lowLatencyCountry, .randomLowLatency:
            return true
        }
    }
}
