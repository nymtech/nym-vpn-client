import ErrorReason
import Foundation
import Testing
import TunnelStatus

struct TunnelStateResponseTests {
    @Test func responseWithoutStateStillDecodes() throws {
        let json = #"{"status":0}"#
        let decoded = try JSONDecoder().decode(TunnelStatusResponse.self, from: Data(json.utf8))
        #expect(decoded.state == nil)
        #expect(decoded.status == .connected)
    }

    @Test func errorStateRoundTrips() throws {
        let response = TunnelStatusResponse(
            status: .disconnected,
            retryAttempt: nil,
            afterDisconnectAction: nil,
            lastError: nil,
            tunnelConnectingState: nil,
            connectionInfoData: nil,
            state: .error(ErrorReason(.needsRelaxedIndependenceCriteria))
        )
        let decoded = try JSONDecoder().decode(TunnelStatusResponse.self, from: JSONEncoder().encode(response))
        guard case let .error(reason) = try #require(decoded.state) else {
            Issue.record("expected the error state")
            return
        }
        #expect(reason.code == .needsRelaxedIndependenceCriteria)
    }

    @Test func connectedAtRoundTripsInsideState() throws {
        let stamp = Date(timeIntervalSince1970: 1_700_000_000)
        let response = TunnelStatusResponse(
            status: .connected,
            retryAttempt: nil,
            afterDisconnectAction: nil,
            lastError: nil,
            tunnelConnectingState: nil,
            connectionInfoData: nil,
            state: .connected(connectionInfo: nil, connectedAt: stamp)
        )
        let encoder = JSONEncoder()
        encoder.dateEncodingStrategy = .iso8601
        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .iso8601
        let decoded = try decoder.decode(TunnelStatusResponse.self, from: encoder.encode(response))
        guard case let .connected(_, connectedAt) = try #require(decoded.state) else {
            Issue.record("expected the connected state")
            return
        }
        #expect(connectedAt == stamp)
    }
}
