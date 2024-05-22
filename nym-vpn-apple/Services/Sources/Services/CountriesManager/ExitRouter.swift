public enum ExitRouter: Codable, Equatable {
    case country(code: String)
    // Fictional, just country under the hood, while we get the actual functionality implemented
    case random

    public var countryCode: String? {
        switch self {
        case let .country(code: countryCode):
            return countryCode
        case .random:
            return nil
        }
    }

    public var isQuickest: Bool {
        switch self {
        case .country, .random:
            return false
        }
    }

    public var isCountry: Bool {
        switch self {
        case .country:
            return true
        case .random:
            return false
        }
    }
}
