import Theme

public enum CredentialsManagerError: Error {
    case noError
    case invalidCredential
    case cannotCreateDB
    case unknownError

    public var localizedTitle: String? {
        switch self {
        case .invalidCredential:
            return "addCredentials.error.invalidCredential".localizedString
        case .noError:
            return nil
        case .cannotCreateDB:
            return "addCredentials.error.cannotCreateDB".localizedString
        case .unknownError:
            return "addCredentials.error.unknownError".localizedString
        }
    }
}
