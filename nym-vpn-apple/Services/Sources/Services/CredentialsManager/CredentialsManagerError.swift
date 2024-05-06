import Theme

public enum CredentialsManagerError: Error, Equatable {
    case noError
    case generalError(String)
    case cannotCreateDB

    public var localizedTitle: String? {
        switch self {
        case .noError:
            return nil
        case .generalError(let text):
            return "\("error".localizedString) \(text)"
        case .cannotCreateDB:
            return "addCredentials.error.cannotCreateDB".localizedString
        }
    }
}
