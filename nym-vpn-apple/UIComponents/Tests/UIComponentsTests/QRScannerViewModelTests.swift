import Foundation
import Testing
@testable import UIComponents

@MainActor
struct QRScannerViewModelTests {
    private let validator = FakeValidator()
    private let clock = FakeClock()

    private func makeViewModel() -> QRScannerViewModel {
        QRScannerViewModel(
            validator: validator,
            defaultInvalidMessage: "invalid",
            invalidMessageDuration: .seconds(60),
            now: { [clock] in clock.now }
        )
    }

    @Test func acceptedCodeIsReturned() async {
        let viewModel = makeViewModel()
        validator.results = [.valid]

        viewModel.onCodeScan("code")
        await viewModel.awaitValidation()

        #expect(viewModel.result == QRScanResult(contents: "code"))
        #expect(viewModel.invalidMessage == nil)
        #expect(!viewModel.isValidating)
    }

    @Test func rejectedCodeShowsValidatorMessage() async {
        let viewModel = makeViewModel()
        validator.results = [.invalid(message: "used")]

        viewModel.onCodeScan("code")
        await viewModel.awaitValidation()

        #expect(viewModel.result == nil)
        #expect(viewModel.invalidMessage == "used")
    }

    @Test func rejectedCodeInViewIsNotValidatedAgain() async {
        let viewModel = makeViewModel()
        validator.results = [.invalid()]

        viewModel.onCodeScan("code")
        await viewModel.awaitValidation()
        viewModel.onCodeScan("code")
        await viewModel.awaitValidation()

        #expect(validator.calls == 1)
        #expect(viewModel.invalidMessage == "invalid")
    }

    @Test func rejectedCodeIsValidatedAgainAfterRetryDelay() async {
        let viewModel = makeViewModel()
        validator.results = [.invalid(retryAfter: .seconds(3)), .valid]

        viewModel.onCodeScan("code")
        await viewModel.awaitValidation()
        clock.advance(by: .seconds(2))
        viewModel.onCodeScan("code")
        #expect(validator.calls == 1)

        clock.advance(by: .seconds(1))
        viewModel.onCodeScan("code")
        await viewModel.awaitValidation()

        #expect(validator.calls == 2)
        #expect(viewModel.result != nil)
    }

    @Test func validatorFailureCanBeRetried() async {
        let viewModel = makeViewModel()
        validator.error = FakeError()

        viewModel.onCodeScan("code")
        await viewModel.awaitValidation()
        #expect(viewModel.invalidMessage == "invalid")

        validator.error = nil
        validator.results = [.valid]
        clock.advance(by: QRScannerViewModel.failureRetryAfter)
        viewModel.onCodeScan("code")
        await viewModel.awaitValidation()

        #expect(viewModel.result != nil)
    }

    @Test func otherCodeIsValidatedAfterRejection() async {
        let viewModel = makeViewModel()
        validator.results = [.invalid(), .valid]

        viewModel.onCodeScan("first")
        await viewModel.awaitValidation()
        viewModel.onCodeScan("second")
        await viewModel.awaitValidation()

        #expect(viewModel.result == QRScanResult(contents: "second"))
    }

    @Test func scansAreIgnoredWhileValidating() async {
        let viewModel = makeViewModel()
        validator.results = [.valid]

        viewModel.onCodeScan("first")
        viewModel.onCodeScan("second")
        await viewModel.awaitValidation()

        #expect(validator.calls == 1)
        #expect(viewModel.result == QRScanResult(contents: "first"))
    }

    @Test func resultDescriptionHidesContents() {
        let result = QRScanResult(contents: "secret")
        #expect(!String(describing: result).contains("secret"))
        #expect(!String(reflecting: result).contains("secret"))
    }
}

@MainActor
private final class FakeValidator: QRScanValidator {
    var results: [QRScanValidation] = []
    var error: Error?
    private(set) var calls = 0

    func validate(_ contents: String) async throws -> QRScanValidation {
        calls += 1
        if let error { throw error }
        return results.isEmpty ? .invalid() : results.removeFirst()
    }
}

private final class FakeClock: @unchecked Sendable {
    private(set) var now = ContinuousClock.now

    func advance(by duration: Duration) {
        now += duration
    }
}

private struct FakeError: Error {}
