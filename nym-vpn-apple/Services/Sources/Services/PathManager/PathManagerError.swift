import Theme

public enum PathManagerError: Error, Equatable {
    case cannotCreateDB
    
    public var localizedTitle: String? {
        switch self {
        case .cannotCreateDB:
            "addCredentials.error.cannotCreateDB".localizedString
        }
    }
}
