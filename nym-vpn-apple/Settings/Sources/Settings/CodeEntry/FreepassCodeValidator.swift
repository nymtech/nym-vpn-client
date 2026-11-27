import CredentialsManager
import UIComponents

@MainActor
final class FreepassCodeValidator: QRScanValidator {
    static let retryAfter: Duration = .seconds(3)

    private let freepassManager: FreepassManager

    init(freepassManager: FreepassManager = .shared) {
        self.freepassManager = freepassManager
    }

    func validate(_ contents: String) async throws -> QRScanValidation {
        guard let code = FreepassCode.parse(contents) else { return .invalid() }
        return switch await freepassManager.redeem(code: code) {
        case .redeemed:
            .valid
        case let .failed(error):
            .invalid(message: error.localizedMessage, retryAfter: error.isFinal ? nil : Self.retryAfter)
        }
    }
}
