import Foundation
import Theme
import UIComponents

@MainActor
@Observable
final class RedeemCodeViewModel {
    private let validator: QRScanValidator
    @ObservationIgnored var onAccepted: ((ScannedQRCode) -> Void)?

    var code = "" {
        didSet {
            guard code != oldValue else { return }
            errorMessage = nil
        }
    }
    private(set) var errorMessage: String?
    private(set) var isLoading = false

    var canSubmit: Bool {
        !isLoading && !code.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
    }

    init(validator: QRScanValidator) {
        self.validator = validator
    }

    func reset() {
        guard !isLoading else { return }
        code = ""
        errorMessage = nil
    }

    func submit() {
        guard canSubmit else { return }
        let contents = code
        guard let scannedCode = ScannedQRCode.parse(contents) else {
            errorMessage = "redeemCode.invalid".localizedString
            return
        }
        isLoading = true
        errorMessage = nil
        // Not tied to the view, so a redeem in flight always finishes.
        Task { [weak self] in
            guard let self else { return }
            let validation: QRScanValidation
            do {
                validation = try await validator.validate(contents)
            } catch {
                validation = .invalid()
            }
            isLoading = false
            switch validation {
            case .valid:
                onAccepted?(scannedCode)
            case let .invalid(message, _):
                errorMessage = message ?? "redeemCode.invalid".localizedString
            }
        }
    }
}
