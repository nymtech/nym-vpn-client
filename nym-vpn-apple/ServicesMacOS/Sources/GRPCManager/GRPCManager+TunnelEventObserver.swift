import NymVPNRpc

extension GRPCManager: TunnelEventObserver {
    nonisolated public func onTunnelEvent(event: TunnelEvent) {
        switch event {
        case let .newState(tunnelState):
            Task { @MainActor [weak self] in
                self?.updateTunnelStatus(with: tunnelState)
            }
        case .mixnetState:
            Task { @MainActor in }
        case .configChanged:
            Task { @MainActor in }
        case .accountState:
            Task { @MainActor in }
        }
    }

    nonisolated public func onClose() {
        Task { @MainActor [weak self] in
            self?.setup()
        }
    }
}
