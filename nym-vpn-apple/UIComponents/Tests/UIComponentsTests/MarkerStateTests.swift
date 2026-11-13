import TunnelStatus
import XCTest
@testable import UIComponents

final class MarkerStateTests: XCTestCase {
    private func marker(_ status: TunnelStatus, killSwitch: Bool = true) -> Marker {
        Marker(tunnelStatus: status, connectingState: nil, isMixnet: false, killSwitch: killSwitch)
    }

    func testStaticStates() {
        XCTAssertEqual(marker(.disconnected).state, .notProtected)
        XCTAssertEqual(marker(.connected).state, .connected)
        XCTAssertEqual(marker(.disconnecting).state, .stopping)
        XCTAssertEqual(marker(.error).state, .error)
    }

    func testOfflineFollowsKillSwitch() {
        XCTAssertEqual(marker(.offline).state, .blocked(.noInternet))
        XCTAssertEqual(marker(.offlineReconnect).state, .blocked(.needsReconnect))
        XCTAssertEqual(marker(.offline, killSwitch: false).state, .notProtected)
        XCTAssertEqual(marker(.offlineReconnect, killSwitch: false).state, .notProtected)
    }

    func testSevenPhasesDrawSixSteps() {
        let expected: [(TunnelConnectingState, Int)] = [
            (.resolvingApiAddresses, 1),
            (.awaitingAccountReadiness, 2),
            (.awaitingCredentialsAvailability, 2),
            (.refreshingGateways, 3),
            (.selectingGateways, 4),
            (.registeringWithGateways, 5),
            (.connectingTunnel, 6)
        ]
        for (phase, step) in expected {
            XCTAssertEqual(Marker.step(for: phase), step, "\(phase)")
        }
    }

    func testModeIsTheFace() {
        let mixnet = Marker(tunnelStatus: .connected, connectingState: nil, isMixnet: true)
        XCTAssertEqual(mixnet.mode, .mixnet)
        XCTAssertEqual(marker(.connected).mode, .dvpn)
    }
}
