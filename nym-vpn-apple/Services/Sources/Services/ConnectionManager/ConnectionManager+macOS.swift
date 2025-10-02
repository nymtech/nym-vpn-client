#if os(macOS)
import Foundation
import TunnelMixnet
import NotificationMessages

extension ConnectionManager {
    @MainActor func connect() async throws {
        try await grpcManager.connect()
        appSettings.statisticsConnectionCount += 1
    }
}

extension ConnectionManager {
    @MainActor public func connectDisconnect() async throws {
        switch grpcManager.tunnelStatus {
        case .connected, .connecting, .offlineReconnect, .error:
            try await grpcManager.disconnect()
        case .disconnected, .disconnecting, .reasserting, .restarting, .offline, .unknown:
            try await connect()
        }
    }

    @MainActor func fetchConnectionConfig() async {
        connectionConfig = await grpcManager.config()
    }

    func updateConnectionConfig() {
        Task {
            guard let connectionConfig else { return }
            try? await grpcManager.updateConfig(newConfig: connectionConfig)
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

        grpcManager.$isServing
            .receive(on: DispatchQueue.main)
            .sink { [weak self] isServing in
                guard isServing else { return }
                Task { @MainActor in
                    await self?.fetchConnectionConfig()
                }
            }
            .store(in: &cancellables)

        grpcManager.$connectionInfoData
            .removeDuplicates()
            .receive(on: DispatchQueue.main)
            .assign(to: \.connectionInfoData, on: self)
            .store(in: &cancellables)

        appSettings.$isQuicEnabledPublisher
            .receive(on: DispatchQueue.main)
            .sink { [weak self] newValue in
                self?.connectionConfig?.enableBridges = newValue
                self?.updateConnectionConfig()
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
