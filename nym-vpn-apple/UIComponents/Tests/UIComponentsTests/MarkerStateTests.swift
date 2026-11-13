import TunnelStatus
import XCTest
@testable import UIComponents

final class MarkerStateTests: XCTestCase {
    private func marker(
        _ status: TunnelStatus,
        connectingState: TunnelConnectingState? = nil,
        killSwitch: Bool = true
    ) -> Marker {
        Marker(tunnelStatus: status, connectingState: connectingState, isMixnet: false, killSwitch: killSwitch)
    }

    func testStaticStates() {
        XCTAssertEqual(marker(.disconnected).state, .notProtected)
        XCTAssertEqual(marker(.unknown).state, .notProtected)
        XCTAssertEqual(marker(.connected).state, .connected)
        XCTAssertEqual(marker(.disconnecting).state, .stopping)
        XCTAssertEqual(marker(.error).state, .error)
    }

    func testOfflineMidSessionWithKillSwitchIsBlocked() {
        XCTAssertEqual(marker(.offlineReconnect).state, .blocked)
    }

    func testOfflineWithNothingToResumeIsNotProtected() {
        XCTAssertEqual(marker(.offline).state, .notProtected)
    }

    func testOfflineWithKillSwitchOffIsNotProtected() {
        XCTAssertEqual(marker(.offlineReconnect, killSwitch: false).state, .notProtected)
    }

    func testConnectingStatusesCarryTheStep() {
        for status: TunnelStatus in [.connecting, .reasserting, .restarting] {
            XCTAssertEqual(marker(status, connectingState: .selectingGateways).state, .connecting(step: 4), "\(status)")
        }
    }

    func testSevenPhasesDrawSixSteps() {
        let expected: [(TunnelConnectingState?, Int)] = [
            (nil, 1),
            (.unrecognized, 1),
            (.resolvingApiAddresses, 1),
            (.awaitingAccountReadiness, 2),
            (.awaitingCredentialsAvailability, 2),
            (.refreshingGateways, 3),
            (.selectingGateways, 4),
            (.registeringWithGateways, 5),
            (.connectingTunnel, 6)
        ]
        for (phase, step) in expected {
            XCTAssertEqual(Marker.step(for: phase), step, "\(String(describing: phase))")
        }
    }

    func testModeIsTheFace() {
        let mixnet = Marker(tunnelStatus: .connected, connectingState: nil, isMixnet: true)
        XCTAssertEqual(mixnet.mode, .mixnet)
        XCTAssertEqual(marker(.connected).mode, .dvpn)
    }
}
