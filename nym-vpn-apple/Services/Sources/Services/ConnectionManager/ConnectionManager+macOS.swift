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

        // If the user has already modified the config, treat this as a returning
        // user — just flip the flag and bail; their settings stay untouched.
        guard connectionStorage.isUsingInitialConfig else {
            appSettings.didCompleteFirstLaunch = true
            return
        }

        await waitUntilDaemonServing()

        guard let daemonConfig = await grpcManager.config() else { return }

        // Adopt daemon config wholesale, then overwrite entry / exit / algorithm
        // with the app's initial values. (Daemon decoder hardcodes the algorithm
        // to `.auto`, so we can't trust it — keep the app's intended value.)
        let initialConfig = connectionStorage.connectionConfig
        var newConfig = daemonConfig
        newConfig.gatewaySelectionAlgorithmConfig = initialConfig.gatewaySelectionAlgorithmConfig

        // Propagate into ConnectionManager (sink persists to ConnectionStorage
        // → AppSettings).
        connectionConfig = newConfig
        connectionType = newConfig.enableTwoHop ? .wireguard : .mixnet5hop
        entryGateway = newConfig.entry
        exitRouter = newConfig.exit

        // Mirror to standalone AppSettings flags.
        appSettings.isLanBypassEnabled = newConfig.allowLan
        appSettings.isIPv6TrafficEnabled = !newConfig.disableIpv6
        appSettings.isAdBlockerEnabled = newConfig.enableAdBlocking
        appSettings.isQuicEnabled = newConfig.enableBridges
        appSettings.customDns = newConfig.dns ?? []
        appSettings.isCustomDnsEnabled = !(newConfig.dns?.isEmpty ?? true)

        // Push the app's entry / exit to the daemon.
        try? await grpcManager.setEntryPoint(newConfig.entry)
        try? await grpcManager.setExitPoint(newConfig.exit)

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
