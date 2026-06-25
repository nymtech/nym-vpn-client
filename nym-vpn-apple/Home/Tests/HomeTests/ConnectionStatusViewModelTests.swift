import XCTest
import ErrorReason
@testable import Home
import ConnectionManager
import TunnelStatus
import UIComponents

@MainActor
final class ConnectionStatusViewModelTests: XCTestCase {
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

    private func makeViewModel() -> ConnectionStatusViewModel {
        ConnectionStatusViewModel(connectionManager: .shared)
    }

    func testIndependenceErrorUsesAwaitingGatewayConsentArc() {
        let viewModel = makeViewModel()
        viewModel.status = .error
        viewModel.lastDisplayedStep = .establishingConnection
        ConnectionManager.shared.lastError = ErrorReason.needsRelaxedIndependenceCriteria

        XCTAssertEqual(viewModel.arcProgressState, .awaitingGatewayConsent)
    }

    func testIndependenceErrorDoesNotUseEstablishingConnectionStepArc() {
        let viewModel = makeViewModel()
        viewModel.status = .error
        viewModel.lastDisplayedStep = .establishingConnection
        ConnectionManager.shared.lastError = ErrorReason.needsRelaxedIndependenceCriteria

        if case .step(.establishingConnection) = viewModel.arcProgressState {
            XCTFail("Independence error must not present establishingConnection arc step")
        }
    }

    func testResumeKeepsReachedStepSoArcDoesNotUnload() {
        // macOS reaches the middle ring before the independence error; the
        // resumed connect must keep that step so rings don't unload.
        XCTAssertEqual(
            ConnectionStatusViewModel.resumedDisplayedStep(current: .choosingBestServers),
            .choosingBestServers
        )
    }

    func testResumeWithNoReachedStepFallsBackToOuterRing() {
        // iOS pre-flight errors before any step is reported; fall back to the
        // outer ring so the consent visual still shows a filled first ring.
        XCTAssertEqual(
            ConnectionStatusViewModel.resumedDisplayedStep(current: nil),
            .authenticatingAccount
        )
    }

    func testClearingIndependenceErrorRemovesAwaitingGatewayConsentArc() {
        let viewModel = makeViewModel()
        viewModel.status = .error
        ConnectionManager.shared.lastError = ErrorReason.needsRelaxedIndependenceCriteria
        XCTAssertEqual(viewModel.arcProgressState, .awaitingGatewayConsent)

        ConnectionManager.shared.lastError = nil
        if case .awaitingGatewayConsent = viewModel.arcProgressState {
            XCTFail("Cleared independence error must not keep awaitingGatewayConsent arc")
        }
    }
}
