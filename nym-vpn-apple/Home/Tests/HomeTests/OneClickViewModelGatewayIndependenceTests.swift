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

    func testIndependenceErrorMapsToAwaitingGatewayConsentConnectState() async {
        let viewModel = makeViewModel()
        ConnectionManager.shared.currentTunnelStatus = .error
        ConnectionManager.shared.lastError = ErrorReason.needsRelaxedIndependenceCriteria
        await Task.yield()
        XCTAssertEqual(viewModel.connectState, .awaitingGatewayConsent)
    }

    func testIndependenceErrorDoesNotMapToStopConnectState() async {
        let viewModel = makeViewModel()
        ConnectionManager.shared.currentTunnelStatus = .error
        ConnectionManager.shared.lastError = ErrorReason.needsRelaxedIndependenceCriteria
        await Task.yield()
        XCTAssertNotEqual(viewModel.connectState, .stop)
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

    func testIndependenceConsentPolicyMatchesDerivedConnectState() async {
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
        XCTAssertEqual(viewModel.connectState, .awaitingGatewayConsent)
    }
}
