import Foundation
import Testing
import TunnelStatus

struct ConnectionTimerAnchorTests {
    @Test func connectedStampIsTheAnchor() {
        let stamp = Date(timeIntervalSince1970: 1_700_000_000)
        #expect(connectionTimerAnchor(isConnected: true, tunnelConnectedAt: stamp) == stamp)
    }

    @Test func connectedWithoutStampHidesTimer() {
        #expect(connectionTimerAnchor(isConnected: true, tunnelConnectedAt: nil) == nil)
    }

    @Test func notConnectedIgnoresStamp() {
        let stamp = Date(timeIntervalSince1970: 1_700_000_000)
        #expect(connectionTimerAnchor(isConnected: false, tunnelConnectedAt: stamp) == nil)
    }

    @Test func laterStampReplacesEarlier() {
        let first = Date(timeIntervalSince1970: 1_700_000_000)
        let second = Date(timeIntervalSince1970: 1_700_000_100)
        #expect(connectionTimerAnchor(isConnected: true, tunnelConnectedAt: first) == first)
        #expect(connectionTimerAnchor(isConnected: true, tunnelConnectedAt: second) == second)
    }
}
