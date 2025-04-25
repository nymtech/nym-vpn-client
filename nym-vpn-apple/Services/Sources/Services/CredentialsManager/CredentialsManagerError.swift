import Theme

public enum CredentialsManagerError: Error, Equatable {
    case noError
    case generalError(String)
    case cannotCreateDB
    case cannotCreateCache
    case noExpiryDate

    public var localizedTitle: String? {
        switch self {
        case .noError:
            nil
        case let .generalError(text):
            "\(text)"
        case .cannotCreateDB:
            "addCredentials.error.cannotCreateDB".localizedString
        case .noExpiryDate:
            "addCredentials.error.noExpiryDate".localizedString
        case .cannotCreateCache:
            "addCredentials.error.cannotCreateCache".localizedString
        }
    }
}
