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
import UIComponents

@MainActor
final class OneClickViewModelGatewayIndependenceTests: XCTestCase {
    private var savedLastError: Error?
    private var savedTunnelStatus: TunnelStatus = .unknown

    override func setUp() {
        super.setUp()
        savedLastError = ConnectionManager.shared.lastError
        savedTunnelStatus = ConnectionManager.shared.currentTunnelStatus
    }

    override func tearDown() {
        ConnectionManager.shared.lastError = savedLastError
        ConnectionManager.shared.currentTunnelStatus = savedTunnelStatus
        super.tearDown()
    }

#if os(iOS)
    private func makeViewModel() -> OneClickViewModel {
        OneClickViewModel(
            appSettings: .shared,
            connectionManager: .shared,
            credentialsManager: .shared,
            gatewayManager: .shared,
            snackbarManager: .shared,
            impactGenerator: .shared,
            networkMonitor: .shared
        )
    }
#elseif os(macOS)
    private func makeViewModel() -> OneClickViewModel {
        OneClickViewModel(
            appSettings: .shared,
            connectionManager: .shared,
            credentialsManager: .shared,
            gatewayManager: .shared,
            snackbarManager: .shared,
            impactGenerator: .shared,
            networkMonitor: .shared,
            grpcManager: .shared
        )
    }
#endif

    // Consent is surfaced by the family-warning modal, not the connect button.
    // The drawer button must stay `.stop` so it is not a second consent surface.
    func testIndependenceErrorMapsToStopConnectState() async {
        let viewModel = makeViewModel()
        ConnectionManager.shared.currentTunnelStatus = .error
        ConnectionManager.shared.lastError = ErrorReason.needsRelaxedIndependenceCriteria
        await Task.yield()
        XCTAssertEqual(viewModel.connectState, .stop)
    }

    func testConnectingTunnelStatusMapsToConnectingConnectState() async {
        let viewModel = makeViewModel()
        ConnectionManager.shared.currentTunnelStatus = .connecting
        ConnectionManager.shared.lastError = nil
        await Task.yield()
        XCTAssertEqual(viewModel.connectState, .connecting)
    }

    func testConnectedTunnelStatusMapsToConnectedConnectState() async {
        let viewModel = makeViewModel()
        ConnectionManager.shared.currentTunnelStatus = .connected
        ConnectionManager.shared.lastError = nil
        await Task.yield()
        XCTAssertEqual(viewModel.connectState, .connected)
    }

    func testDisconnectFromErrorDoesNotConnectWhenAlreadyDisconnected() async {
        let viewModel = makeViewModel()
        ConnectionManager.shared.currentTunnelStatus = .disconnected
        ConnectionManager.shared.lastError = nil
        viewModel.disconnectFromError()
        await Task.yield()
        XCTAssertNotEqual(viewModel.connectState, .connecting)
    }

    // The consent error is still preserved (so the arc shows the consent state
    // and the modal stays actionable), but the connect button stays `.stop`.
    func testIndependenceConsentIsPreservedButButtonStaysStop() async {
        let viewModel = makeViewModel()
        ConnectionManager.shared.currentTunnelStatus = .error
        ConnectionManager.shared.lastError = ErrorReason.needsRelaxedIndependenceCriteria
        await Task.yield()
        XCTAssertTrue(
            GatewayIndependenceArcPolicy.shouldPreserveIndependenceConsentError(
                status: .error,
                lastError: ErrorReason.needsRelaxedIndependenceCriteria
            )
        )
        XCTAssertEqual(viewModel.connectState, .stop)
    }
}
