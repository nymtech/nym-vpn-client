#if os(iOS)
import UIComponents

/// The scanner accepts any code, so each one goes to the validator for its kind.
@MainActor
final class ScannedCodeRouter: QRScanValidator {
    private let pairing: QRScanValidator
    private let freepass: QRScanValidator

    init() {
        pairing = PairingQrValidator()
        freepass = FreepassCodeValidator()
    }

    func validate(_ contents: String) async throws -> QRScanValidation {
        switch ScannedQRCode.parse(contents) {
        case .pairing:
            try await pairing.validate(contents)
        case .freepass:
            try await freepass.validate(contents)
        case nil:
            .invalid()
        }
    }
}
#endif
