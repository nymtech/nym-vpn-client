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

    @Test func connectionInfoFromAnOlderPayloadHasNoAmnezia() throws {
        let legacy = Data(#"{"entryGatewayId":"a","exitGatewayId":"b","tunnelType":"wireguard"}"#.utf8)
        let decoded = try JSONDecoder().decode(ConnectionInfoData.self, from: legacy)
        #expect(decoded.amneziaVersion == nil)
        #expect(decoded.amneziaViaTranslator == nil)

        let current = ConnectionInfoData(
            entryGatewayId: "a",
            exitGatewayId: "b",
            amneziaVersion: "3.1",
            amneziaViaTranslator: true
        )
        #expect(try JSONDecoder().decode(ConnectionInfoData.self, from: JSONEncoder().encode(current)) == current)
    }
}
