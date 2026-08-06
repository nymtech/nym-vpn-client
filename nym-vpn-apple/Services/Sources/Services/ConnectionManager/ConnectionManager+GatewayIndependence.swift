import Foundation
import TunnelStatus
#if os(macOS)
import GRPCManager
#endif

@MainActor
extension ConnectionManager {
    /// Called after the user accepts the relaxed-independence warning. Relaxes
    /// gateway independence; the library reconnects by itself from the error
    /// state. On iOS the message also releases the extension pre-flight wait
    /// (`startTunnel` suspends until consent); on a disconnected macOS tunnel
    /// the connect is kicked off explicitly.
    public func acceptRelaxedGatewayIndependence() async throws {
#if os(iOS)
        try await tunnelsManager.activeTunnel?.send(.setGatewayIndependence(false))
#elseif os(macOS)
        try await grpcManager.setGatewayIndependence(false)
        // The daemon does not auto-reconnect after relaxing, so kick the
        // connect from the error (or disconnected) state explicitly.
        guard currentTunnelStatus == .disconnected || currentTunnelStatus == .error else { return }
        // From the error state the daemon throttles the relaxed-criteria push
        // by ~1s; wait it out so the connect re-selects with relaxed settings
        // instead of the stale strict ones (disconnected applies immediately,
        // so no wait needed there).
        // STOPGAP: remove once core pushes the setting immediately
        // (vpn_service `handle_set_enable_gateway_independence`).
        if currentTunnelStatus == .error {
            try? await Task.sleep(for: .seconds(1.2))
        }
        try await startTunnel()
#endif
    }
}
