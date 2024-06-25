public enum ExitRouter: Codable, Equatable {
    case country(code: String)

    public var countryCode: String? {
        switch self {
        case let .country(code: countryCode):
            return countryCode
        }
    }

    public var isQuickest: Bool {
        switch self {
        case .country:
            return false
        }
    }

    public var isCountry: Bool {
        switch self {
        case .country:
            return true
        }
    }
}
