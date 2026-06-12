import XCTest
@testable import Home
import CredentialsManager
import UIComponents

@MainActor
final class HomeTests: XCTestCase {
    override func setUp() {
        super.setUp()
        OnboardingSession.shared.reset()
    }

    func testProcessingFailureUsesStableRetryCopy() async throws {
        struct PreparationError: Error {}

        let viewModel = ProcessingAccountViewModel(
            flow: .login,
            prepareAccount: { _ in throw PreparationError() }
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
            prepareAccount: { _ in
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
        OnboardingSession.shared.beginCarouselSession()
        let viewModel = ProcessingAccountViewModel(
            flow: .login,
            prepareAccount: { _ in }
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

    func testProcessingExitsEarlyWhenWorkCompletesBeforeCarousel() async throws {
        let viewModel = ProcessingAccountViewModel(
            flow: .login,
            prepareAccount: { _ in }
        )

        viewModel.start()
        try await Task.sleep(for: .milliseconds(50))
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
            prepareAccount: { canPrefetch in
                XCTAssertFalse(canPrefetch)
            }
        )

        viewModel.start()
        try await Task.sleep(for: .milliseconds(50))
        XCTAssertNil(viewModel.errorMessage)
    }

    func testProcessingCompletesWhenAccountInactive() async throws {
        let viewModel = ProcessingAccountViewModel(
            flow: .login,
            prepareAccount: { _ in }
        )

        viewModel.start()
        try await Task.sleep(for: .milliseconds(50))
        XCTAssertNil(viewModel.errorMessage)

        viewModel.animationDidFinish()
        XCTAssertTrue(viewModel.didShowFinalMessage)
    }

    func testOnboardingSessionDoesNotRegressPhase() {
        OnboardingSession.shared.advance(to: .processingComplete)
        OnboardingSession.shared.advance(to: .registered)
        XCTAssertEqual(OnboardingSession.shared.phase, .processingComplete)
    }

    func testPurchasePresentedOnlyOnce() {
        XCTAssertTrue(OnboardingSession.shared.shouldPresentPurchase)
        OnboardingSession.shared.markPurchaseFlowPresented()
        XCTAssertFalse(OnboardingSession.shared.shouldPresentPurchase)
    }

    func testCancelPurchaseFlowAllowsRetry() {
        OnboardingSession.shared.markPurchaseFlowPresented()
        XCTAssertFalse(OnboardingSession.shared.shouldPresentPurchase)
        OnboardingSession.shared.cancelPurchaseFlow()
        XCTAssertTrue(OnboardingSession.shared.shouldPresentPurchase)
    }

    func testPostPurchaseModeRequiresActiveSubscription() {
        XCTAssertEqual(ProcessingFlow.postPurchase.processingMode, .postPurchase)
        XCTAssertEqual(ProcessingFlow.createAccount.processingMode, .prePurchase)
        XCTAssertEqual(ProcessingFlow.login.processingMode, .prePurchase)
    }

    func testAuthRegistrationBlockedAfterProcessingComplete() {
        OnboardingSession.shared.advance(to: .processingComplete)
        XCTAssertFalse(OnboardingSession.shared.canStartProcessing)
    }

    func testCreateAccountFlowUsesThreeCarouselSteps() {
        XCTAssertEqual(ProcessingFlow.createAccount.carouselStepCount, 3)
    }

    func testPostPurchaseFlowUsesFourCarouselSteps() {
        XCTAssertEqual(ProcessingFlow.postPurchase.carouselStepCount, 4)
    }

    func testCarouselStepSyncsWithAccountSetupPhase() {
        let viewModel = ProcessingAccountViewModel(
            flow: .login,
            prepareAccount: { _ in }
        )
        viewModel.syncCarouselStep(for: .syncingSummary)
        XCTAssertEqual(viewModel.currentStep, 2)
        viewModel.syncCarouselStep(for: .fetchingTickets)
        XCTAssertEqual(viewModel.currentStep, 3)
    }

    func testPostPurchaseCarouselStepSyncsWithFetchingTicketsPhase() {
        let viewModel = ProcessingAccountViewModel(
            flow: .postPurchase,
            prepareAccount: { _ in }
        )
        viewModel.syncCarouselStep(for: .fetchingTickets)
        XCTAssertEqual(viewModel.currentStep, 4)
    }

    func testCarouselStepMappingForAllPhases() {
        XCTAssertNil(ProcessingAccountViewModel.carouselStep(for: .idle, flow: .login))
        XCTAssertEqual(ProcessingAccountViewModel.carouselStep(for: .syncingSummary, flow: .createAccount), 2)
        XCTAssertEqual(ProcessingAccountViewModel.carouselStep(for: .fetchingTickets, flow: .postPurchase), 4)
        XCTAssertEqual(ProcessingAccountViewModel.carouselStep(for: .ready, flow: .login), 3)
    }

    func testPurchaseOnlyEntrySkipsRegistrationContract() {
        XCTAssertFalse(OnboardingLaunchPolicy.shouldRegisterAccountOnLaunch(displayPurchaseView: true))
    }
}
