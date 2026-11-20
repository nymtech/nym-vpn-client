import CredentialsManager
import Theme
import UIComponents

@MainActor
final class ScannedCodeValidator: QRScanValidator {
    static let retryAfter: Duration = .seconds(3)

#if os(iOS)
    private let pairingManager: PairingManager
#endif
    private let freepassManager: FreepassManager

#if os(iOS)
    init(pairingManager: PairingManager = .shared, freepassManager: FreepassManager = .shared) {
        self.pairingManager = pairingManager
        self.freepassManager = freepassManager
    }
#elseif os(macOS)
    init(freepassManager: FreepassManager = .shared) {
        self.freepassManager = freepassManager
    }
#endif

    func validate(_ contents: String) async throws -> QRScanValidation {
        switch ScannedQRCode.parse(contents) {
#if os(iOS)
        case let .pairing(publicKey):
            await validatePairing(publicKey)
#elseif os(macOS)
        case .pairing:
            .invalid()
#endif
        case let .freepass(code):
            await validateFreepass(code)
        case nil:
            .invalid()
        }
    }

#if os(iOS)
    private func validatePairing(_ publicKey: String) async -> QRScanValidation {
        switch await pairingManager.onScannedKey(publicKey) {
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
#endif

    private func validateFreepass(_ code: String) async -> QRScanValidation {
        switch await freepassManager.redeem(code: code) {
        case .redeemed:
            .valid
        case let .failed(error):
            .invalid(message: error.localizedMessage, retryAfter: error.isFinal ? nil : Self.retryAfter)
        }
    }
}
