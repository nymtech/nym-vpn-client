import Foundation
import Theme

@MainActor
@Observable
final class QRScannerViewModel {
    // A validator failure says nothing about the code, so it can be retried.
    static let failureRetryAfter: Duration = .seconds(3)

    private let validator: QRScanValidator
    private let defaultInvalidMessage: String
    private let invalidMessageDuration: Duration
    private let now: () -> ContinuousClock.Instant

    private(set) var isValidating = false
    private(set) var invalidMessage: String?
    private(set) var result: QRScanResult?

    // Stops a rejected code still in view from being re-validated on every frame.
    @ObservationIgnored private var lastRejection: Rejection?
    @ObservationIgnored private var hideInvalidMessageTask: Task<Void, Never>?
    @ObservationIgnored private var validationTask: Task<Void, Never>?

    fileprivate struct Rejection {
        let contents: String
        let message: String
        let retryAt: ContinuousClock.Instant?
    }

    init(
        validator: QRScanValidator,
        defaultInvalidMessage: String = "qrScanner.invalid".localizedString,
        invalidMessageDuration: Duration = .seconds(2),
        now: @escaping () -> ContinuousClock.Instant = { ContinuousClock.now }
    ) {
        self.validator = validator
        self.defaultInvalidMessage = defaultInvalidMessage
        self.invalidMessageDuration = invalidMessageDuration
        self.now = now
    }

    func onCodeScan(_ contents: String) {
        guard result == nil, !isValidating else { return }
        if let lastRejection, lastRejection.contents == contents, !canRetry(lastRejection) {
            showInvalidMessage(lastRejection.message)
            return
        }
        isValidating = true
        invalidMessage = nil
        validationTask = Task { [weak self] in
            guard let self else { return }
            let validation = await validate(contents)
            isValidating = false
            switch validation {
            case .valid:
                hideInvalidMessageTask?.cancel()
                lastRejection = nil
                invalidMessage = nil
                result = QRScanResult(contents: contents)
            case let .invalid(message, retryAfter):
                let message = message ?? defaultInvalidMessage
                lastRejection = Rejection(
                    contents: contents,
                    message: message,
                    retryAt: retryAfter.map { self.now() + $0 }
                )
                showInvalidMessage(message)
            }
        }
    }

    func awaitValidation() async {
        await validationTask?.value
    }

    func cancelPendingWork() {
        hideInvalidMessageTask?.cancel()
    }
}

private extension QRScannerViewModel {
    func canRetry(_ rejection: Rejection) -> Bool {
        guard let retryAt = rejection.retryAt else { return false }
        return now() >= retryAt
    }

    func validate(_ contents: String) async -> QRScanValidation {
        do {
            return try await validator.validate(contents)
        } catch {
            return .invalid(retryAfter: Self.failureRetryAfter)
        }
    }

    func showInvalidMessage(_ message: String) {
        invalidMessage = message
        hideInvalidMessageTask?.cancel()
        hideInvalidMessageTask = Task { [weak self, invalidMessageDuration] in
            try? await Task.sleep(for: invalidMessageDuration)
            guard !Task.isCancelled else { return }
            self?.invalidMessage = nil
        }
    }
}
