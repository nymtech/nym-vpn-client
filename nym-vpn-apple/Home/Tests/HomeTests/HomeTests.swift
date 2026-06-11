import XCTest
@testable import Home

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
}
