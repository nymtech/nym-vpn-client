public enum ExitRouter: Codable, Equatable {
    case country(code: String)
    case gateway(String)

    public var countryCode: String? {
        switch self {
        case let .country(code: countryCode):
            countryCode
        case .gateway:
            nil
        }
    }

    public var isQuickest: Bool {
        switch self {
        case .country, .gateway:
            return false
        }
    }

    public var isCountry: Bool {
        switch self {
        case .country:
            true
        case .gateway:
            false
        }
    }
}
