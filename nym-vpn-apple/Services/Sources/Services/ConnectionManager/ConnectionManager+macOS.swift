#if os(macOS)
import TunnelMixnet
import NotificationMessages

extension ConnectionManager {
    func generateConfig() -> MixnetConfig {
        var config = MixnetConfig(
            entryGateway: entryGateway,
            exitRouter: exitRouter,
            isZknymEnabled: appSettings.isZknymEnabled
        )

        switch connectionType {
        case .mixnet5hop:
            config = MixnetConfig(
                entryGateway: entryGateway,
                exitRouter: exitRouter,
                isTwoHopEnabled: false,
                isZknymEnabled: appSettings.isZknymEnabled
            )
        case .wireguard:
            config = MixnetConfig(
                entryGateway: entryGateway,
                exitRouter: exitRouter,
                isTwoHopEnabled: true,
                isZknymEnabled: appSettings.isZknymEnabled
            )
        }
        return config
    }

    @MainActor func connect(with config: MixnetConfig) async throws {
        appSettings.lastConnectionIntent = config.toJson()
        try await grpcManager.connect(
            entryGateway: config.entryGateway,
            exitRouter: config.exitRouter,
            isTwoHopEnabled: config.isTwoHopEnabled,
            isZknymEnabled: appSettings.isZknymEnabled
        )
    }

    /// Sends connect command to deamon if entry/exit gateways changed while connected,
    /// to initiate reconnect
    @MainActor func reconnectIfNeeded() async {
        let newConfig = generateConfig()
        guard currentTunnelStatus == .connected, newConfig.toJson() != appSettings.lastConnectionIntent else { return }
        do {
            try await connect(with: newConfig)
        } catch {
            lastError = error
        }
    }
}

extension ConnectionManager {
    @MainActor public func connectDisconnect() async throws {
        let config = generateConfig()

        if grpcManager.tunnelStatus == .connected
            || grpcManager.tunnelStatus == .connecting
            || grpcManager.tunnelStatus == .offlineReconnect {
            grpcManager.disconnect()
        } else {
            try await connect(with: config)
        }
    }
}

// MARK: - Setup -
extension ConnectionManager {
    func setupGRPCManagerObservers() {
        grpcManager.$tunnelStatus.sink { [weak self] status in
            Task { @MainActor [weak self] in
                guard self?.currentTunnelStatus != status else { return }
                self?.currentTunnelStatus = status
                self?.scheduleNotificationIfNeeded()
                self?.updateTimeConnected()
            }
        }
        .store(in: &cancellables)
    }
}

// MARK: - Time connected -
extension ConnectionManager {
    func updateTimeConnected() {
        guard grpcManager.tunnelStatus == .connected,
              let newConnectedDate = grpcManager.connectedDate
        else {
            connectedDate = nil
            return
        }
        self.connectedDate = newConnectedDate
    }
}

// MARK: - Notification -
private extension ConnectionManager {
    func scheduleNotificationIfNeeded() {
        guard currentTunnelStatus == .disconnecting else { return }
        Task {
            await NotificationMessages.scheduleDisconnectNotification()
        }
    }
}
#endif
