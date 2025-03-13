#if os(iOS)
import NetworkExtension
import AppSettings
import MixnetLibrary
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
                    self?.currentTunnelStatus = status
                    self?.updateTimeConnected()
                }
            }
    }
}

// MARK: - Connection -
extension ConnectionManager {
    func generateConfig() throws -> MixnetConfig {
        do {
            let credentialURL = try credentialsManager.dataFolderURL()
            var config = MixnetConfig(
                entryGateway: entryGateway,
                exitRouter: exitRouter,
                credentialsDataPath: credentialURL.path(),
                isZknymEnabled: appSettings.isZknymEnabled
            )

            switch connectionType {
            case .mixnet5hop:
                config = MixnetConfig(
                    entryGateway: entryGateway,
                    exitRouter: exitRouter,
                    credentialsDataPath: credentialURL.path(),
                    isTwoHopEnabled: false,
                    isZknymEnabled: appSettings.isZknymEnabled
                )
            case .wireguard:
                config = MixnetConfig(
                    entryGateway: entryGateway,
                    exitRouter: exitRouter,
                    credentialsDataPath: credentialURL.path(),
                    isTwoHopEnabled: true,
                    isZknymEnabled: appSettings.isZknymEnabled
                )
            }
            return config
        } catch let error {
            throw error
        }
    }

    @MainActor func connect(with config: MixnetConfig) async throws {
        do {
            try await tunnelsManager.loadTunnels()
            let tunnel = try await tunnelsManager.addUpdate(tunnelConfiguration: config, isOndemandEnabled: true)
            activeTunnel = tunnel
            try await tunnelsManager.connect(tunnel: tunnel)
        } catch {
            throw error
        }
    }

    /// Sends connect command to lib if entry/exit gateways changed while connected,
    /// to initiate reconnect
    @MainActor func reconnectIfNeeded() async {
        do {
            let newConfig = try generateConfig()
            guard currentTunnelStatus == .connected,
                  let tunnelProviderProtocol = activeTunnel?.tunnel.protocolConfiguration as? NETunnelProviderProtocol,
                  let mixnetConfig = tunnelProviderProtocol.asMixnetConfig(),
                  newConfig.toJson() != mixnetConfig.toJson()
            else {
                return
            }
            try await connectDisconnect(isAutoConnect: true)
        } catch {
            lastError = error
        }
    }

    func disconnectActiveTunnel() {
        guard let activeTunnel,
              shouldDisconnectActiveTunnel()
        else {
            return
        }
        activeTunnel.tunnel.isOnDemandEnabled = false
        activeTunnel.tunnel.saveToPreferences()
        tunnelsManager.disconnect(tunnel: activeTunnel)
        Task {
            try await tunnelsManager.loadTunnels()
        }
    }

    func shouldDisconnectActiveTunnel() -> Bool {
        guard let activeTunnel else { return false }

        switch activeTunnel.status {
        case .connected, .connecting, .reasserting, .restarting, .offlineReconnect:
            return true
        case .disconnecting, .disconnected, .offline, .unknown:
            return false
        }
    }
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
    /// - Parameter isAutoConnect: Bool.
    /// true - when reconnecting automatically, after change of connection settings:  country(UK, DE) or type(5hop, 2hop...).
    /// false - when user manually taps "Connect".
    /// On reconnect, after disconnect, the connectDisconnect is called as a user tapped connect.
    @MainActor public func connectDisconnect(isAutoConnect: Bool = false) async throws {
        do {
            let config = try generateConfig()
            isReconnecting = isReconnecting(newConfig: config)
            if isReconnecting {
                // Reconnecting after change of country, 5hop...
                disconnectActiveTunnel()
            } else {
                // User "Connect" button actions
                guard !isAutoConnect else { return }
                if shouldDisconnectActiveTunnel() {
                    isDisconnecting = true
                    disconnectActiveTunnel()
                } else {
                    try await connect(with: config)
                }
            }
        } catch let error {
            throw error
        }
    }

    func updateTunnelStatusIfReconnecting() {
        guard isReconnecting,
              currentTunnelStatus == .disconnected
        else {
            return
        }
        isReconnecting = false
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) { [weak self] in
            Task {
                try? await self?.connectDisconnect()
            }
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
