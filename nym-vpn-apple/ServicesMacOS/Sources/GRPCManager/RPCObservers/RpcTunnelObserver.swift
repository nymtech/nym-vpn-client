import NymVPNRpc

final class RpcTunnelObserver: TunnelEventObserver {
    func onTunnelEvent(event: TunnelEvent) {
        print("RpcTunnelObserver: event: \(event)")
    }

    func onClose() {
        print("RpcTunnelObserver: closed!!!")
    }
}
