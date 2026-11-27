import CredentialsManager
import Theme

extension PairingError {
    var localizedMessage: String {
        switch self {
        case .offline:
            "pairing.failed.offline".localizedString
        case .notSignedIn:
            "pairing.failed.notSignedIn".localizedString
        case .alreadySignedIn:
            "pairing.error.alreadySignedIn".localizedString
        case .vpnConnected:
            "pairing.error.vpnConnected".localizedString
        case .sessionEnded:
            "pairing.ended".localizedString
        case .invalidCode, .server, .unknown:
            "pairing.failed".localizedString
        }
    }
}
