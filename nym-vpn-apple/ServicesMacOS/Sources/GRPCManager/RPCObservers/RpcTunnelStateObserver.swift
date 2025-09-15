import NymVPNRpc

final class RpcTunnelStateObserver: TunnelStateObserver {
    func onTunnelStateChange(newState: TunnelState) {
        print("RPCTunnelStateObserver: tuneel state changed: \(newState)")

        Task {
//            self.updateTunnelStatus(with: newState)
        }
    }

    func onClose() {
//        resetTunnelStateChangeObserver()
        print("RPCTunnelStateObserver: tunnel state did close")
    }
}
