#if os(macOS)
import Foundation
import TunnelMixnet
import NotificationMessages

extension ConnectionManager {
    func generateConfig() -> MixnetConfig {
        let isErrorReportingEnabled = appSettings.currentEnv == "sandbox" ? true : appSettings.isErrorReportingOn

        switch connectionType {
        case .mixnet5hop:
            return MixnetConfig(
                entryGateway: entryGateway,
                exitRouter: exitRouter,
                isErrorReportingEnabled: isErrorReportingEnabled,
                isStatisticsEnabled: appSettings.isStatisticsEnabled,
                isTwoHopEnabled: false
            )
        case .wireguard:
            return MixnetConfig(
                entryGateway: entryGateway,
                exitRouter: exitRouter,
                isErrorReportingEnabled: isErrorReportingEnabled,
                isStatisticsEnabled: appSettings.isStatisticsEnabled,
                isTwoHopEnabled: true
            )
        }
    }

    @MainActor func connect(with config: MixnetConfig) async throws {
        appSettings.lastConnectionIntent = config.toJson()
        try await grpcManager.connect(
            entryGateway: config.entryGateway,
            exitRouter: config.exitRouter,
            isTwoHopEnabled: config.isTwoHopEnabled,
            disableIPv6: !appSettings.isIPv6TrafficEnabled
        )
        appSettings.statisticsConnectionCount += 1
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

        switch grpcManager.tunnelStatus {
        case .connected, .connecting, .offlineReconnect, .error:
            try await grpcManager.disconnect()
        case .disconnected, .disconnecting, .reasserting, .restarting, .offline, .unknown:
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

        grpcManager.$connectionRetryAttempt
            .receive(on: DispatchQueue.main)
            .sink { [weak self] attempt in
                MainActor.assumeIsolated {
                    self?.connectionRetryAttempt = attempt
                }
            }
            .store(in: &cancellables)

        grpcManager.$tunnelConnectingState
            .receive(on: DispatchQueue.main)
            .sink { [weak self] newState in
                MainActor.assumeIsolated {
                    self?.tunnelConnectingState = newState
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
