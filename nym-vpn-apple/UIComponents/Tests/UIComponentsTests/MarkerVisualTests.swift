import TunnelStatus
import XCTest
@testable import UIComponents

final class MarkerVisualTests: XCTestCase {
    func testArcStepsDrawTheMarkerSteps() {
        let steps: [(ArcProgressState.Step, TunnelConnectingState)] = [
            (.initializingNym, .resolvingApiAddresses),
            (.authenticatingAccount, .awaitingAccountReadiness),
            (.downloadingZkNyms, .awaitingCredentialsAvailability),
            (.updatingServerList, .refreshingGateways),
            (.choosingBestServers, .selectingGateways),
            (.registeringWithServers, .registeringWithGateways),
            (.establishingConnection, .connectingTunnel)
        ]
        for (arcStep, connectingState) in steps {
            XCTAssertEqual(arcStep.markerStep, Marker.step(for: connectingState), "\(arcStep)")
        }
    }

    func testBlinkRunsBetweenLowAndHighOverThePeriod() {
        XCTAssertEqual(MarkerBlink.opacity(at: 0), 0.15, accuracy: 0.0001)
        XCTAssertEqual(MarkerBlink.opacity(at: 1.2), 0.9, accuracy: 0.0001)
        XCTAssertEqual(MarkerBlink.opacity(at: 2.4), 0.15, accuracy: 0.0001)
        for time in stride(from: 0.0, to: 4.8, by: 0.1) {
            let opacity = MarkerBlink.opacity(at: time)
            XCTAssertGreaterThanOrEqual(opacity, 0.15 - 0.0001)
            XCTAssertLessThanOrEqual(opacity, 0.9 + 0.0001)
        }
    }
}
