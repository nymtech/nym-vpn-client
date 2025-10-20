import NymVPNRpc
import Combine

final class RPCTunnelObserver: ObservableObject, TunnelEventObserver, @unchecked Sendable {
    @Published var tunnelEvent: TunnelEvent?
    @Published var didClose = false

    func onTunnelEvent(event: TunnelEvent) {
        tunnelEvent = event
    }

    func onClose() {
        didClose = true
    }
}
