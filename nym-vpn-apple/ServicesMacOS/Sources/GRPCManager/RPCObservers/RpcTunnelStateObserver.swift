import NymVPNRpc

final class RPCTunnelStateObserver: TunnelEventObserver {
    func onTunnelEvent(event: TunnelEvent) {
        print("onTuneEvent \(event)")
    }
    
    func onClose() {
        print("onClose()")
    }
}
