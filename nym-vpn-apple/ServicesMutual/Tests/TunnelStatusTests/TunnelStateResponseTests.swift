import ErrorReason
import Foundation
import Testing
import TunnelStatus

struct TunnelStateResponseTests {
    @Test func errorStateRoundTrips() throws {
        let state = TunnelStateResponse.error(ErrorReason(.needsRelaxedIndependenceCriteria))
        let decoded = try JSONDecoder().decode(TunnelStateResponse.self, from: JSONEncoder().encode(state))
        guard case let .error(reason) = decoded else {
            Issue.record("expected the error state")
            return
        }
        #expect(reason.code == .needsRelaxedIndependenceCriteria)
    }

    @Test func connectedAtRoundTripsAsIso8601() throws {
        let stamp = Date(timeIntervalSince1970: 1_700_000_000)
        let encoder = JSONEncoder()
        encoder.dateEncodingStrategy = .iso8601
        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .iso8601
        let state = TunnelStateResponse.connected(connectionInfo: nil, connectedAt: stamp)
        let decoded = try decoder.decode(TunnelStateResponse.self, from: encoder.encode(state))
        #expect(decoded.connectedAt == stamp)
    }
}
