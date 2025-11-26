#if os(iOS)
import NetworkExtension
import AppSettings
import Constants
import CredentialsManager
import NymVPNLib
import TunnelMixnet
import Tunnels

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
        let credentialURL = try CredentialsManager.dataFolderURL()
        let configURL = try CredentialsManager.configFolderURL()

        switch connectionType {
        case .mixnet5hop:
            return MixnetConfig(
                entryGateway: entryGateway,
                exitRouter: exitRouter,
                credentialsDataPath: credentialURL.path(),
                configPath: configURL.path(),
                isErrorReportingEnabled: isErrorReportingEnabled,
                isStatisticsEnabled: appSettings.isStatisticsEnabled,
                isQuicEnabled: appSettings.isQuicEnabled,
                isLanBypassEnabled: appSettings.isLanBypassEnabled,
                isTwoHopEnabled: false
            )
        case .wireguard:
            return MixnetConfig(
                entryGateway: entryGateway,
                exitRouter: exitRouter,
                credentialsDataPath: credentialURL.path(),
                configPath: configURL.path(),
                isErrorReportingEnabled: isErrorReportingEnabled,
                isStatisticsEnabled: appSettings.isStatisticsEnabled,
                isQuicEnabled: appSettings.isQuicEnabled,
                isLanBypassEnabled: appSettings.isLanBypassEnabled,
                isTwoHopEnabled: true
            )
        }
    }

    @MainActor func connect(with config: MixnetConfig) async throws {
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

    /// Sends connect command to lib if entry/exit gateways changed while connected,
    /// to initiate reconnect
    func reconnectIfNeeded() async {
        do {
            let newConfig = try generateConfig()
            guard currentTunnelStatus == .connected || currentTunnelStatus == .connecting,
                  let tunnelProviderProtocol = activeTunnel?.tunnel.protocolConfiguration as? NETunnelProviderProtocol,
                  let mixnetConfig = tunnelProviderProtocol.asMixnetConfig(),
                  newConfig.toJson() != mixnetConfig.toJson(),
                  !isReconnecting
            else {
                return
            }
            isReconnecting = true
            try await disconnectActiveTunnel()
            await waitForTunnelStatus(with: .disconnected)
            try await connectDisconnect()
            isReconnecting = false
        } catch {
            lastError = error
        }
    }

    func disconnectActiveTunnel() async throws {
        guard let activeTunnel,
              shouldDisconnectActiveTunnel()
        else {
            return
        }
        if !isReconnecting {
            activeTunnel.tunnel.isOnDemandEnabled = false
            try await activeTunnel.saveToPreferencesAndLoadTunnels()
        }
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

    // Placeholders after rpcClient
    func updateConnectionConfig() {
        Task { @MainActor in
            await reconnectIfNeeded()
        }
    }

    func fetchConnectionConfig() async {}
}

extension ConnectionManager {
// TODO: use this once iOS tunnel supports tunnel reconnection
//    @MainActor public func connectDisconnect() async throws {
//        do {
//            let config = try generateConfig()
//
//            if shouldDisconnectActiveTunnel() {
//                disconnectActiveTunnel()
//            } else {
//                try await connect(with: config)
//            }
//        } catch let error {
//            throw error
//        }
//    }
}

// TODO: remove extension once tunnel supports reconnect
extension ConnectionManager {
    /// connects disconnects VPN, depending on current VPN status
    @MainActor public func connectDisconnect() async throws {
        do {
            if shouldDisconnectActiveTunnel() {
                isDisconnecting = true
                try await disconnectActiveTunnel()
                lastError = nil
            } else {
                let config = try generateConfig()
                try await connect(with: config)
            }
        } catch let error {
            throw error
        }
    }

    func isReconnecting(newConfig: MixnetConfig) -> Bool {
        guard let tunnelProviderProtocol = activeTunnel?.tunnel.protocolConfiguration as? NETunnelProviderProtocol,
              let mixnetConfig = tunnelProviderProtocol.asMixnetConfig(),
              currentTunnelStatus == .connected, newConfig != mixnetConfig
        else {
            return false
        }
        return true
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
