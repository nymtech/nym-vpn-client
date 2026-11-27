#if os(iOS)
import CredentialsManager
import Theme
import UIComponents

@MainActor
final class PairingQrValidator: QRScanValidator {
    static let retryAfter: Duration = .seconds(3)

    private let pairingManager: PairingManager

    init(pairingManager: PairingManager = .shared) {
        self.pairingManager = pairingManager
    }

    func validate(_ contents: String) async throws -> QRScanValidation {
        guard let publicKey = PairingQrCode.parsePublicKey(contents) else { return .invalid() }
        return switch await pairingManager.onScannedKey(publicKey) {
        case .success:
            .valid
        case .failure(.invalidCode):
            .invalid()
        case .failure(.alreadySignedIn):
            .invalid(message: "pairing.error.alreadySignedIn".localizedString)
        case .failure(.offline):
            .invalid(message: "pairing.error.offlineRetrying".localizedString, retryAfter: Self.retryAfter)
        case .failure(.server):
            .invalid(message: "pairing.error.serverRetrying".localizedString, retryAfter: Self.retryAfter)
        case .failure(.vpnConnected):
            .invalid(message: "pairing.error.vpnConnected".localizedString, retryAfter: Self.retryAfter)
        case .failure(.notSignedIn), .failure(.sessionEnded), .failure(.unknown):
            .invalid(message: "pairing.error.genericRetrying".localizedString, retryAfter: Self.retryAfter)
        }
    }
}
#endif
