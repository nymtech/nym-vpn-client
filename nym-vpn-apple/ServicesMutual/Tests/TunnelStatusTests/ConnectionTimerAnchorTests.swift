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

    @Test func connectedAtRoundTripsAsIso8601() throws {
        let stamp = Date(timeIntervalSince1970: 1_700_000_000)
        let response = TunnelStatusResponse(
            status: .connected,
            retryAttempt: nil,
            afterDisconnectAction: nil,
            lastError: nil,
            tunnelConnectingState: nil,
            connectionInfoData: nil,
            connectedAt: stamp
        )
        let encoder = JSONEncoder()
        encoder.dateEncodingStrategy = .iso8601
        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .iso8601
        let decoded = try decoder.decode(TunnelStatusResponse.self, from: encoder.encode(response))
        #expect(decoded.connectedAt == stamp)
    }
}
