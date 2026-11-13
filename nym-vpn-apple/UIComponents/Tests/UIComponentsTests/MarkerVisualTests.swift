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

    func testFaceIsStillWithoutATick() {
        XCTAssertNil(MarkerFaceModel.dvpnHop(tick: nil))
        XCTAssertNil(MarkerFaceModel.mixHop(tick: nil))
        XCTAssertEqual(MarkerFaceModel.mixRouteIndex(tick: nil), 0)
        for element in 0..<3 {
            XCTAssertEqual(MarkerFaceModel.dvpnOpacity(element: element, tick: nil), 1)
        }
        for index in 0..<5 {
            XCTAssertEqual(MarkerFaceModel.mixHopOpacity(index: index, tick: nil), 1)
        }
    }

    func testDvpnCyclesEntryLinkExitEveryThreeTicks() {
        let hops = (0..<10).map { MarkerFaceModel.dvpnHop(tick: $0) }
        XCTAssertEqual(hops, [0, 0, 0, 1, 1, 1, 2, 2, 2, 0])
        XCTAssertEqual(MarkerFaceModel.dvpnOpacity(element: 1, tick: 3), 1)
        XCTAssertEqual(MarkerFaceModel.dvpnOpacity(element: 0, tick: 3), MarkerFaceModel.dim)
    }

    func testMixnetPassIsFiveHopsAndARest() {
        let hops = stride(from: 0, to: 14, by: 2).map { MarkerFaceModel.mixHop(tick: $0) }
        XCTAssertEqual(hops, [0, 1, 2, 3, 4, 5, 0])
        // tick 4 lights hop 2: passed hops trail, upcoming hops dim
        XCTAssertEqual(MarkerFaceModel.mixHopOpacity(index: 2, tick: 4), 1)
        XCTAssertEqual(MarkerFaceModel.mixHopOpacity(index: 1, tick: 4), MarkerFaceModel.trail)
        XCTAssertEqual(MarkerFaceModel.mixHopOpacity(index: 3, tick: 4), MarkerFaceModel.dim)
        // the rest beat leaves every hop passed
        XCTAssertEqual(MarkerFaceModel.mixHopOpacity(index: 4, tick: 10), MarkerFaceModel.trail)
    }

    func testMixnetRouteChangesEveryTwoPasses() {
        // two passes of six beats, two ticks per beat
        XCTAssertEqual(MarkerFaceModel.mixRouteIndex(tick: 23), 0)
        XCTAssertEqual(MarkerFaceModel.mixRouteIndex(tick: 24), 1)
        XCTAssertEqual(MarkerFaceModel.mixRouteIndex(tick: 72), 3)
        XCTAssertEqual(MarkerFaceModel.mixRouteIndex(tick: 96), 0)
    }

    func testTickCountsWholeIntervalsSincePlayStarted() {
        let start = Date(timeIntervalSinceReferenceDate: 0)
        XCTAssertEqual(MarkerFaceModel.tick(from: start, to: start, interval: 0.08), 0)
        XCTAssertEqual(MarkerFaceModel.tick(from: start, to: start.addingTimeInterval(0.24), interval: 0.08), 3)
        XCTAssertEqual(MarkerFaceModel.tick(from: start, to: start.addingTimeInterval(-1), interval: 0.08), 0)
    }

    func testMixnetRoutesRunEntryThroughEachLayerToExit() {
        for route in MarkerFaceModel.mixRoutes {
            XCTAssertEqual(route.count, 5)
            XCTAssertEqual(route.first, MarkerFaceModel.mixEntry)
            XCTAssertEqual(route.last, MarkerFaceModel.mixExit)
            XCTAssertEqual(route[1...3].map(\.x), MarkerFaceModel.layerX)
        }
    }
}
