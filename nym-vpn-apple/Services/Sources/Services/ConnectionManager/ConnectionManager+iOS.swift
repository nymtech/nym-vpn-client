#if os(iOS)
import NetworkExtension
import AppSettings
import ConfigurationManager
import ConnectionTypes
import NymLogger
import NymVPNLib
import PathManager
import TunnelMixnet
import Tunnels
import TunnelStatus

// MARK: - Setup -
extension ConnectionManager {
    func setupTunnelManagerObservers() {
        tunnelsManager.$isLoaded
            .receive(on: DispatchQueue.main)
            .sink { [weak self] isLoaded in
                MainActor.assumeIsolated {
                    self?.isTunnelManagerLoaded = isLoaded
                }
            }
            .store(in: &cancellables)

        tunnelsManager.$activeTunnel
            .receive(on: DispatchQueue.main)
            .sink { [weak self] tunnel in
                MainActor.assumeIsolated {
                    self?.activeTunnel = tunnel
                }
            }
            .store(in: &cancellables)
    }

    func configureTunnelStatusObserver(tunnel: Tunnel) {
        tunnelStatusUpdateCancellable = tunnel.$status
            .receive(on: DispatchQueue.main)
            .sink { [weak self] status in
                MainActor.assumeIsolated {
                    if status == .disconnecting, let afterDisconnectAction = self?.afterDisconnectAction {
                        switch afterDisconnectAction {
                        case .offline:
                            self?.currentTunnelStatus = .offline
                        case .reconnect:
                            self?.currentTunnelStatus = .connecting
                        }
                    }
                    self?.currentTunnelStatus = status
                    self?.updateTimeConnected()
                }
            }

        tunnelRetryAttemptCancellable = tunnel.$retryAttempt
            .receive(on: DispatchQueue.main)
            .sink { [weak self] attempt in
                MainActor.assumeIsolated {
                    self?.connectionRetryAttempt = attempt
                }
            }

        tunnelAfterRetryCancellable = tunnel.$afterDisconnectAction
            .receive(on: DispatchQueue.main)
            .sink { [weak self] action in
                MainActor.assumeIsolated {
                    self?.afterDisconnectAction = action
                }
            }

        tunnelLastErrorCancelable = tunnel.$lastError
            .receive(on: DispatchQueue.main)
            .sink { [weak self] newError in
                MainActor.assumeIsolated {
                    self?.lastError = newError
                }
            }

        tunnelConnectingStateCancellable = tunnel.$tunnelConnectingState
            .receive(on: DispatchQueue.main)
            .sink { [weak self] newState in
                MainActor.assumeIsolated {
                    self?.tunnelConnectingState = newState
                }
            }

        tunnelConnectionInfoDataCancellable = tunnel.$connectionInfoData
            .removeDuplicates()
            .receive(on: DispatchQueue.main)
            .assign(to: \.connectionInfoData, on: self)
    }
}

// MARK: - Connection -
extension ConnectionManager {
    func generateConfig() throws -> MixnetConfig {
        let isErrorReportingEnabled = appSettings.currentEnv == "sandbox" ? true : appSettings.isErrorReportingOn
        let dataURL = try PathManager.dataFolderURL()
        let configURL = try PathManager.configFolderURL()
        guard let logsURL = LogFileManager.logsDirectory() else {
            throw PathManagerError.cannotCreateDB
        }
        let algorithmConfig = connectionConfig.gatewaySelectionAlgorithmConfig

        return MixnetConfig(
            entryGateway: entryGateway,
            exitRouter: exitRouter,
            configPath: configURL.path(),
            dataPath: dataURL.path(),
            logPath: logsURL.path(),
            customDns: appSettings.isCustomDnsEnabled ? appSettings.customDns : [],
            mixnetTuning: connectionConfig.mixnetTuningConfig,
            isErrorReportingEnabled: isErrorReportingEnabled,
            isStatisticsEnabled: appSettings.isStatisticsEnabled,
            isQuicEnabled: appSettings.isQuicEnabled,
            isStealthApiEnabled: appSettings.isStealthApiEnabled,
            isLanBypassEnabled: appSettings.isLanBypassEnabled,
            isAdBlockingEnabled: appSettings.isAdBlockerEnabled,
            isTwoHopEnabled: connectionType == .wireguard,
            gatewaySelectionAlgorithmConfig: algorithmConfig,
            isServerFamilyRemindersEnabled: appSettings.serverFamilyRemindersEnabled
        )
    }

    @MainActor func connect(with config: MixnetConfig) async throws {
        credentialsManager.shutdownControllers()
        do {
            try await tunnelsManager.loadTunnels()
            let tunnel = try await tunnelsManager.addUpdate(tunnelConfiguration: config, isOndemandEnabled: true)
            activeTunnel = tunnel
            try await tunnelsManager.connect(tunnel: tunnel)
            appSettings.statisticsConnectionCount += 1
        } catch {
            throw error
        }
    }

    func disconnectActiveTunnel() async throws {
        guard let activeTunnel,
              shouldDisconnectActiveTunnel()
        else {
            return
        }
        activeTunnel.tunnel.isOnDemandEnabled = false
        try await activeTunnel.saveToPreferencesAndLoadTunnels()
        tunnelsManager.disconnect(tunnel: activeTunnel)
    }

    func shouldDisconnectActiveTunnel() -> Bool {
        guard let activeTunnel else { return false }

        switch activeTunnel.status {
        case .connected, .connecting, .reasserting, .restarting, .offlineReconnect, .error:
            return true
        case .disconnecting, .disconnected, .offline, .unknown:
            return false
        }
    }

    func fetchConnectionConfig() async {}

    @MainActor
    func sendAfterPersistingConfig(_ message: TunnelProviderMessage) async throws {
        if let cfg = try? generateConfig() {
            MixnetConfigStorage.save(cfg)
        }
        try await tunnelsManager.send(message: message)
    }

    @MainActor
    public func runDiagnostic() async -> String? {
        guard let environment = ConfigurationManager.shared.networkEnv else { return nil }
        return try? await NymVPNLib.runDiagnostic(
            params: DiagnosticRunParams(
                gateway: nil,
                skipDns: false,
                skipHttp: false,
                skipHybridTransport: false
            ),
            environment: environment
        )
    }
}

extension ConnectionManager {
    /// connects disconnects VPN, depending on current VPN status
    @MainActor public func connectDisconnect() async throws {
        if shouldDisconnectActiveTunnel() {
            isDisconnecting = true
            try await disconnectActiveTunnel()
            if !GatewayIndependenceArcPolicy.isIndependenceConsentError(lastError) {
                lastError = nil
            }
        } else {
            let config = try generateConfig()
            try await connect(with: config)
        }
    }
}

// MARK: - Connection Time -
extension ConnectionManager {
    func updateTimeConnected() {
        guard let activeTunnel = self.activeTunnel,
              activeTunnel.status == .connected,
              let newConnectedDate = activeTunnel.tunnel.connection.connectedDate
        else {
            connectedDate = nil
            return
        }
        connectedDate = newConnectedDate
    }
}
#endif
