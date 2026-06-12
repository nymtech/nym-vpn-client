import Theme

public enum CredentialsManagerError: Error, Equatable {
    case noError
    case generalError(String)
    case subscriptionVerifying
    case cannotCreateDB
    case cannotCreateCache
    case noExpiryDate

    public var localizedTitle: String? {
        switch self {
        case .noError:
            nil
        case let .generalError(text):
            "\(text)"
        case .subscriptionVerifying:
            "processingAccount.subtitle2".localizedString
        case .cannotCreateDB:
            "addCredentials.error.cannotCreateDB".localizedString
        case .noExpiryDate:
            "addCredentials.error.noExpiryDate".localizedString
        case .cannotCreateCache:
            "addCredentials.error.cannotCreateCache".localizedString
        }
    }
}
