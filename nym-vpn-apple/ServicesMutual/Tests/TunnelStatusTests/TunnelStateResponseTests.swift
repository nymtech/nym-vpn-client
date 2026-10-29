import ErrorReason
import Foundation
import Testing
import TunnelStatus

struct TunnelStateResponseTests {
    @Test func errorStateIsTheErrorStatus() {
        #expect(TunnelStatus(.error(ErrorReason(.needsRelaxedIndependenceCriteria))) == .error)
    }

    @Test(arguments: [(true, TunnelStatus.offlineReconnect), (false, TunnelStatus.offline)])
    func offlineKeepsReconnect(reconnect: Bool, expected: TunnelStatus) {
        #expect(TunnelStatus(.offline(reconnect: reconnect)) == expected)
    }

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
}
