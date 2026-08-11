#if os(macOS)
import Foundation
import ConnectionTypes
import Constants
import NotificationMessages
import TunnelMixnet
import TunnelStatus
import WidgetKit

extension ConnectionManager {
    @MainActor func connect() async throws {
        // Reminders off means the user opted out of the warning: relax up front
        // so the daemon never surfaces `needsRelaxedIndependenceCriteria` (no modal).
        try? await grpcManager.setGatewayIndependence(appSettings.serverFamilyRemindersEnabled)
        try await startTunnel()
    }

    @MainActor func startTunnel() async throws {
        try await grpcManager.connect()
        appSettings.statisticsConnectionCount += 1
    }
}

extension ConnectionManager {
    @MainActor public func connectDisconnect() async throws {
        if MockMode.isEnabled {
            if currentTunnelStatus == .connected || currentTunnelStatus == .connecting {
                MockConnectionState.shared.disconnect()
            } else {
                MockConnectionState.shared.connect()
            }
            return
        }
        switch grpcManager.tunnelStatus {
        case .connected, .connecting, .offlineReconnect, .error:
            try await grpcManager.disconnect()
        case .disconnected, .disconnecting, .reasserting, .restarting, .offline, .unknown:
            try await connect()
        }
    }
}

// MARK: - Setup -

extension ConnectionManager {
    func setupGRPCManagerObservers() {
        updateWidgetState(for: currentTunnelStatus)

        grpcManager.$isServing.sink { [weak self] isConnectedToDaemon in
            guard isConnectedToDaemon else { return }

            Task { @MainActor [weak self] in
                await self?.fetchDaemonConfig()
            }
        }
        .store(in: &cancellables)

        grpcManager.$tunnelStatus.sink { [weak self] status in
            Task { @MainActor [weak self] in
                guard self?.currentTunnelStatus != status else { return }
                self?.currentTunnelStatus = status
                self?.scheduleNotificationIfNeeded()
                self?.updateTimeConnected()
                self?.updateWidgetState(for: status)
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

        grpcManager.$connectionInfoData
            .removeDuplicates()
            .receive(on: DispatchQueue.main)
            .assign(to: \.connectionInfoData, on: self)
            .store(in: &cancellables)
    }

    @MainActor private func fetchDaemonConfig() async {
        guard let daemonConfig = await grpcManager.config() else { return }

        let preservedCustomAppPaths = connectionConfig.splitTunnelConfig.customAppPaths
        connectionConfig = daemonConfig
        connectionConfig.splitTunnelConfig.customAppPaths = preservedCustomAppPaths
        connectionType = daemonConfig.enableTwoHop ? .wireguard : .mixnet5hop
        entryGateway = daemonConfig.entry
        exitRouter = daemonConfig.exit

        appSettings.isLanBypassEnabled = daemonConfig.allowLan
        appSettings.isIPv6TrafficEnabled = !daemonConfig.disableIpv6
        appSettings.isAdBlockerEnabled = daemonConfig.enableAdBlocking
        appSettings.isQuicEnabled = daemonConfig.enableBridges
        appSettings.customDns = daemonConfig.dns ?? []
        appSettings.isCustomDnsEnabled = !(daemonConfig.dns?.isEmpty ?? true)
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

// MARK: - Widget -
extension ConnectionManager {
    func updateWidgetState(for status: TunnelStatus) {
        let defaults = UserDefaults(suiteName: Constants.groupID.rawValue)
        if status == .connected {
            if let code = connectionStorage.entryGateway.countryCode {
                let name = Locale.current.localizedString(forRegionCode: code) ?? code
                defaults?.set(name, forKey: "macos_widgetEntryLocation")
            }
            if let code = connectionStorage.exitRouter.countryCode {
                let name = Locale.current.localizedString(forRegionCode: code) ?? code
                defaults?.set(name, forKey: "macos_widgetExitLocation")
            }
        }
        defaults?.set(status.rawValue, forKey: "macos_widgetTunnelStatus")
        WidgetCenter.shared.reloadAllTimelines()
    }
}
#endif
