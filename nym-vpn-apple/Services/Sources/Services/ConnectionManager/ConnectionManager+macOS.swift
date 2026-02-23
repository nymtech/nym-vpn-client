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

// MARK: - First-launch bootstrap -
extension ConnectionManager {
    /// First-launch reconciliation with the daemon.
    ///
    /// Runs only when:
    ///   1. `appSettings.didCompleteFirstLaunch == false`, and
    ///   2. the stored ConnectionConfig is byte-identical to the freshly
    ///      generated initial config (i.e. the user hasn't touched anything).
    ///
    /// Replaces the local config with the daemon's, then forces `entry` and
    /// `exit` back to the app's initial values and pushes them to the daemon.
    /// Sets the flag so subsequent launches no-op.
    @MainActor public func bootstrapFromDaemonIfNeeded() async {
        guard !appSettings.didCompleteFirstLaunch else { return }

        guard connectionStorage.isUsingInitialConfig
        else {
            appSettings.didCompleteFirstLaunch = true
            return
        }

        await waitUntilDaemonServing()

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

        appSettings.didCompleteFirstLaunch = true
    }

    @MainActor private func waitUntilDaemonServing() async {
        guard !grpcManager.isServing else { return }
        for await serving in grpcManager.$isServing.values where serving {
            return
        }
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
