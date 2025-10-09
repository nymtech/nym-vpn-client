import NymVPNRpc
import Combine

@MainActor
final class RPCTunnelObserver: ObservableObject, TunnelEventObserver, @unchecked Sendable {
    @Published var tunnelEvent: TunnelEvent?
    @Published var didClose = false

    public func onTunnelEvent(event: TunnelEvent) {
        tunnelEvent = event
    }

    public func onClose() {
        didClose = true
    }
}
