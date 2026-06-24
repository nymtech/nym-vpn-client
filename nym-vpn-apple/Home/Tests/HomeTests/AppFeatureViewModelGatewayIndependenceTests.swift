import XCTest
import ErrorReason
@testable import Home
import AppSettings
import ConnectionManager
import CredentialsManager
import GatewayManager
import ImpactGenerator
import NetworkMonitor
import SnackbarManager
import TunnelStatus
#if os(macOS)
import GRPCManager
#endif

@MainActor
final class AppFeatureViewModelGatewayIndependenceTests: XCTestCase {
    private var savedLastError: Error?
    private var savedTunnelStatus: TunnelStatus = .unknown
    private var savedRemindersEnabled = true

    override func setUp() {
        super.setUp()
        savedLastError = ConnectionManager.shared.lastError
        savedTunnelStatus = ConnectionManager.shared.currentTunnelStatus
        savedRemindersEnabled = AppSettings.shared.serverFamilyRemindersEnabled
    }

    override func tearDown() {
        ConnectionManager.shared.lastError = savedLastError
        ConnectionManager.shared.currentTunnelStatus = savedTunnelStatus
        AppSettings.shared.serverFamilyRemindersEnabled = savedRemindersEnabled
        super.tearDown()
    }

    private func makeViewModel() -> AppFeatureViewModel {
#if os(iOS)
        AppFeatureViewModel(
            appSettings: .shared,
            credentialsManager: .shared,
            connectionManager: .shared,
            gatewayManager: .shared,
            snackbarManager: .shared,
            impactGenerator: .shared,
            networkMonitor: .shared
        )
#elseif os(macOS)
        AppFeatureViewModel(
            appSettings: .shared,
            credentialsManager: .shared,
            connectionManager: .shared,
            gatewayManager: .shared,
            snackbarManager: .shared,
            impactGenerator: .shared,
            networkMonitor: .shared,
            grpcManager: .shared
        )
#endif
    }

    func testWatcherShowsFamilyWarningModalWhenRemindersEnabled() async {
        AppSettings.shared.serverFamilyRemindersEnabled = true
        ConnectionManager.shared.currentTunnelStatus = .error
        ConnectionManager.shared.lastError = ErrorReason.needsRelaxedIndependenceCriteria

        let viewModel = makeViewModel()
        await Task.yield()

        XCTAssertTrue(viewModel.isFamilyWarningModalDisplayed)
    }

    func testWatcherDoesNotShowModalWhenRemindersDisabled() async {
        AppSettings.shared.serverFamilyRemindersEnabled = false
        ConnectionManager.shared.currentTunnelStatus = .error
        ConnectionManager.shared.lastError = ErrorReason.needsRelaxedIndependenceCriteria

        let viewModel = makeViewModel()
        await Task.yield()

        XCTAssertFalse(viewModel.isFamilyWarningModalDisplayed)
    }

    func testConnectionFailedIndependenceErrorShowsModalWhenRemindersEnabled() async {
        AppSettings.shared.serverFamilyRemindersEnabled = true
        ConnectionManager.shared.currentTunnelStatus = .error
        ConnectionManager.shared.lastError = ErrorReason.needsRelaxedIndependenceCriteria

        let viewModel = makeViewModel()
        viewModel.connectionStatus.onConnectionFailed?("needs-relaxed-independence-criteria")
        await Task.yield()

        XCTAssertTrue(viewModel.isFamilyWarningModalDisplayed)
    }
}
