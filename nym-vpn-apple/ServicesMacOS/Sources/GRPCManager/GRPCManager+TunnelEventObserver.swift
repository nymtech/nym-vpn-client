import NymVPNRpc

extension GRPCManager: TunnelEventObserver {
    public func onTunnelEvent(event: TunnelEvent) {
        switch event {
        case let .newState(tunnelState):
            Task { @MainActor in
                updateTunnelStatus(with: tunnelState)
            }
        case let .mixnetState(mixnetEvent):
            print("mixnet event: \(mixnetEvent)")
        case let .configChanged(boxedVpnServiceConfig):
            print("configChanged: \(boxedVpnServiceConfig)")
        case let .accountState(accountControllerState):
            print("accountControllerState: \(accountControllerState)")
        }
    }

    public func onClose() {
        setup()
    }
}
