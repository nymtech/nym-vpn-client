import XCTest
@testable import Home
import UIComponents

@MainActor
final class HomeTests: XCTestCase {
    func testProcessingFailureUsesStableRetryCopy() async throws {
        struct PreparationError: Error {}

        let viewModel = ProcessingAccountViewModel(
            flow: .login,
            prepareAccountForConnection: { _ in throw PreparationError() }
        )

        viewModel.start()
        try await Task.sleep(for: .milliseconds(50))

        XCTAssertEqual(
            viewModel.errorMessage,
            "generalNymError.somethingWentWrong".localizedString
        )
    }

    func testRetryResetsTerminalProcessingStateBeforeRestarting() async throws {
        let prepareStarted = expectation(description: "prepare restarted")
        prepareStarted.expectedFulfillmentCount = 2

        let viewModel = ProcessingAccountViewModel(
            flow: .login,
            prepareAccountForConnection: { _ in
                prepareStarted.fulfill()
            }
        )

        viewModel.start()
        try await Task.sleep(for: .milliseconds(50))
        viewModel.animationDidAdvance()
        viewModel.animationDidFinish()
        XCTAssertTrue(viewModel.didShowFinalMessage)

        viewModel.retry()

        XCTAssertEqual(viewModel.currentStep, 1)
        XCTAssertFalse(viewModel.didFinishAnimatingText)
        XCTAssertFalse(viewModel.didShowFinalMessage)
        XCTAssertNil(viewModel.errorMessage)

        await fulfillment(of: [prepareStarted], timeout: 1)
    }

    func testRetryRotatesTitlesSessionID() {
        let viewModel = ProcessingAccountViewModel(
            flow: .login,
            prepareAccountForConnection: { _ in }
        )
        let original = viewModel.titlesSessionID
        viewModel.retry()
        XCTAssertNotEqual(viewModel.titlesSessionID, original)
    }

    func testProcessingCarouselUsesFourSecondPacing() {
        XCTAssertEqual(
            ProcessingAccountViewModel.processingStepInterval,
            SwitchingTitlesView.accountProcessingStepInterval
        )
        XCTAssertEqual(SwitchingTitlesView.accountProcessingStepInterval, 4.0)
    }

    func testAdvanceIfReadyWaitsForBothPrepAndCarousel() async throws {
        let viewModel = ProcessingAccountViewModel(
            flow: .login,
            prepareAccountForConnection: { _ in
                try await Task.sleep(for: .milliseconds(100))
            }
        )

        viewModel.start()
        viewModel.animationDidFinish()
        XCTAssertFalse(viewModel.didShowFinalMessage)

        try await Task.sleep(for: .milliseconds(150))
        XCTAssertTrue(viewModel.didShowFinalMessage)
    }

    func testPassphraseLoginRegistersBeforeAuthComplete() async throws {
        var events: [String] = []
        let viewModel = PassphraseSignInViewModel(
            addCredential: { _ in events.append("add") },
            registerAccount: { events.append("register") }
        )
        viewModel.onAuthComplete = { events.append("authComplete") }
        viewModel.passphraseText = "seed phrase"
        viewModel.loginButtonTapped()
        try await Task.sleep(for: .milliseconds(50))
        XCTAssertEqual(events, ["add", "register", "authComplete"])
    }

    func testGeneratePassphraseRegistersBeforeAuthComplete() async throws {
        var events: [String] = []
        let viewModel = GeneratePassphraseViewModel(
            isValidCredentialImported: { true },
            registerAccount: { events.append("register") }
        )
        viewModel.onAuthComplete = { events.append("authComplete") }
        viewModel.start()
        try await Task.sleep(for: .milliseconds(50))
        XCTAssertEqual(events, ["register", "authComplete"])
        XCTAssertTrue(viewModel.didRegisterAccount)
    }

    func testProcessingSucceedsWhenPrefetchGateBlocks() async throws {
        let viewModel = ProcessingAccountViewModel(
            flow: .login,
            canPrefetchZkNyms: { false },
            prepareAccountForConnection: { canPrefetch in
                XCTAssertFalse(canPrefetch)
            }
        )

        viewModel.start()
        try await Task.sleep(for: .milliseconds(50))
        XCTAssertNil(viewModel.errorMessage)
    }
}
