import CredentialsManager
import Theme

extension FreepassError {
    var localizedMessage: String {
        switch self {
        case .invalidCode:
            "freepass.error.invalid".localizedString
        case .alreadyRedeemed:
            "freepass.error.alreadyRedeemed".localizedString
        case .rateLimited:
            "freepass.error.rateLimited".localizedString
        case .offline:
            "freepass.error.offline".localizedString
        case .subscriptionActive:
            "freepass.error.subscriptionActive".localizedString
        case .vpnConnected:
            "freepass.error.vpnConnected".localizedString
        case .accountNotRegistered, .server, .unknown:
            "freepass.error.generic".localizedString
        }
    }
}
