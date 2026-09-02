import XCTest
@testable import Home
import AppSettings
import ConnectionManager
import CredentialsManager
import ErrorReason
import GatewayManager
import ImpactGenerator
import NetworkMonitor
import SnackbarManager
import Theme
import TunnelStatus

@MainActor
private final class SessionCoordinatorSpy: AppSessionCoordinating {
    private(set) var actions: [CoordinatorAction] = []

    func handle(_ action: CoordinatorAction) {
        actions.append(action)
    }
}

@MainActor
final class OneClickViewModelCTATapTests: XCTestCase {
    private var snackbarManager: SnackbarManager!

    override func setUp() {
        super.setUp()
        snackbarManager = SnackbarManager()
    }

    override func tearDown() {
        snackbarManager.clear()
        snackbarManager = nil
        super.tearDown()
    }

    private func makeViewModel() -> OneClickViewModel {
#if os(iOS)
        OneClickViewModel(
            appSettings: .shared,
            connectionManager: .shared,
            credentialsManager: .shared,
            gatewayManager: .shared,
            snackbarManager: snackbarManager,
            impactGenerator: .shared,
            networkMonitor: .shared
        )
#elseif os(macOS)
        OneClickViewModel(
            appSettings: .shared,
            connectionManager: .shared,
            credentialsManager: .shared,
            gatewayManager: .shared,
            snackbarManager: snackbarManager,
            impactGenerator: .shared,
            networkMonitor: .shared,
            grpcManager: .shared
        )
#endif
    }

    func testNoAccountTapRequestsWelcome() {
        let viewModel = makeViewModel()
        let spy = SessionCoordinatorSpy()
        viewModel.sessionCoordinator = spy
        viewModel.connectState = .noAccount

        XCTAssertTrue(viewModel.handleDisconnectedHomeCTATap())
        XCTAssertEqual(spy.actions, [.requestWelcome])
    }

    func testInactiveAccountTunnelErrorRequestsPlanPurchase() {
        let viewModel = makeViewModel()
        let spy = SessionCoordinatorSpy()
        viewModel.sessionCoordinator = spy
        let previousStatus = viewModel.connectionManager.currentTunnelStatus
        let previousError = viewModel.connectionManager.lastError
        defer {
            viewModel.connectionManager.currentTunnelStatus = previousStatus
            viewModel.connectionManager.lastError = previousError
        }

        viewModel.connectionManager.currentTunnelStatus = .error
        viewModel.connectionManager.lastError = ErrorReason.inactiveAccount
        viewModel.handleInactiveSubscriptionErrorIfNeeded()

        XCTAssertEqual(spy.actions, [.requestInactiveSubscriptionPurchase])
    }

    func testNoSubscriptionTapRequestsPlanPurchase() {
        let viewModel = makeViewModel()
        let spy = SessionCoordinatorSpy()
        viewModel.sessionCoordinator = spy
        viewModel.connectState = .noSubscription

        XCTAssertTrue(viewModel.handleDisconnectedHomeCTATap())
        XCTAssertEqual(spy.actions, [.requestInactiveSubscriptionPurchase])
    }

    func testCheckingAccountTapDoesNotRouteCoordinatorOrConnect() {
        let viewModel = makeViewModel()
        let spy = SessionCoordinatorSpy()
        viewModel.sessionCoordinator = spy
        viewModel.connectState = .checkingAccount

        XCTAssertTrue(viewModel.handleDisconnectedHomeCTATap())
        XCTAssertTrue(spy.actions.isEmpty)
    }

    func testDisconnectedTapDoesNotRouteCoordinator() {
        let viewModel = makeViewModel()
        let spy = SessionCoordinatorSpy()
        viewModel.sessionCoordinator = spy
        viewModel.connectState = .disconnected

        XCTAssertFalse(viewModel.handleDisconnectedHomeCTATap())
        XCTAssertTrue(spy.actions.isEmpty)
    }

    func testStopTapIsNotConsumedByDisconnectedHomeCTA() {
        let viewModel = makeViewModel()
        let spy = SessionCoordinatorSpy()
        viewModel.sessionCoordinator = spy
        viewModel.connectState = .stop

        XCTAssertFalse(viewModel.handleDisconnectedHomeCTATap())
        XCTAssertTrue(spy.actions.isEmpty)
    }

    func testAccountUnreachableTapStartsRefreshAndIgnoresReentry() async {
        let viewModel = makeViewModel()
        let spy = SessionCoordinatorSpy()
        viewModel.sessionCoordinator = spy
        viewModel.connectState = .accountUnreachable

        XCTAssertTrue(viewModel.handleDisconnectedHomeCTATap())
        XCTAssertTrue(viewModel.isRefreshingAccountSummary)
        XCTAssertNotNil(viewModel.accountSummaryRefreshTask)
        XCTAssertTrue(spy.actions.isEmpty)

        XCTAssertTrue(viewModel.handleDisconnectedHomeCTATap())
        XCTAssertTrue(viewModel.isRefreshingAccountSummary)
        XCTAssertTrue(spy.actions.isEmpty)

        await viewModel.accountSummaryRefreshTask?.value
        XCTAssertFalse(viewModel.isRefreshingAccountSummary)
    }

    func testAccountUnreachableRetryFailedShowsSnackbarWithRetry() async {
        let viewModel = makeViewModel()
        viewModel.connectState = .accountUnreachable

        viewModel.presentAccountUnreachableRetryFailed()

        XCTAssertEqual(snackbarManager.current?.title, "home.accountUnreachable".localizedString)
        XCTAssertEqual(snackbarManager.current?.message, "error.unexpected".localizedString)
        XCTAssertEqual(snackbarManager.current?.actionTitle, "retry".localizedString)

        snackbarManager.current?.onAction?()
        XCTAssertTrue(viewModel.isRefreshingAccountSummary)
        viewModel.accountSummaryRefreshTask?.cancel()
        await viewModel.accountSummaryRefreshTask?.value
    }
}
