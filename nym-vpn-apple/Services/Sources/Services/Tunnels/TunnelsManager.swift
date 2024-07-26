import Combine
import NetworkExtension
import Logging
import Keychain

public final class TunnelsManager: ObservableObject {
    public static let shared = TunnelsManager()

    @Published public var isLoaded: Result<Void, Error>?
    @Published public var activeTunnel: Tunnel?
    public var tunnels = [Tunnel]()
    public var logger = Logger(label: "TunnelsManager")

    private var cancellables = Set<AnyCancellable>()

    init() {
        Task {
            try? await loadTunnels()
            observeTunnelStatuses()
        }
    }
}

// MARK: - Management -
extension TunnelsManager {
    public func loadTunnels() async throws {
        do {
            let loadedTunnels = try await loadAllTunnelManagers()
            activeTunnel = loadedTunnels.first { $0.tunnel.isEnabled }
            tunnels = loadedTunnels
            isLoaded = .success(())
        } catch {
            logger.log(level: .error, "Failed loading tunnel managers with \(error)")
            isLoaded = .failure(error)
            throw error
        }
    }
}

// MARK: - Connection -
extension TunnelsManager {
    public func connect(tunnel: Tunnel) {
        guard tunnels.contains(tunnel) else { return } // Ensure it's not deleted
        guard tunnel.status == .disconnected
        else {
            // activationDelegate?.tunnelActivationAttemptFailed(tunnel: tunnel, error: .tunnelIsNotInactive)
            return
        }

        //        if let alreadyWaitingTunnel = tunnels.first(where: { $0.status == .waiting }) {
        //            alreadyWaitingTunnel.status = .disconnected
        //        }

        //        if let tunnelInOperation = tunnels.first(where: { $0.status != .disconnected }) {
        //            wg_log(.info, message: "Tunnel '\(tunnel.name)' waiting for deactivation of '\(tunnelInOperation.name)'")
        //            tunnel.status = .waiting
        //            activateWaitingTunnelOnDeactivation(of: tunnelInOperation)
        //            if tunnelInOperation.status != .deactivating {
        //                if tunnelInOperation.isActivateOnDemandEnabled {
        //                    setOnDemandEnabled(false, on: tunnelInOperation) { [weak self] error in
        //                        guard error == nil else {
        //                            wg_log(.error, message: "Unable to activate tunnel '\(tunnel.name)' because on-demand could not be disabled on active tunnel '\(tunnel.name)'")
        //                            return
        //                        }
        //                        self?.startDeactivation(of: tunnelInOperation)
        //                    }
        //                } else {
        //                    startDeactivation(of: tunnelInOperation)
        //                }
        //            }
        //            return
        //        }

        #if targetEnvironment(simulator)
            tunnel.status = .connected
        #else
            tunnel.connect()
        #endif
    }

    public func disconnect(tunnel: Tunnel) {
        // tunnel.isAttemptingActivation = false
        guard tunnel.status != .disconnected && tunnel.status != .disconnecting else { return }
        #if targetEnvironment(simulator)
            tunnel.status = .disconnected
        #else
            tunnel.disconnect()
        #endif
    }
}

private extension TunnelsManager {
    func loadAllTunnelManagers() async throws -> [Tunnel] {
        do {
            var tunnelManagers = try await NETunnelProviderManager.loadAllFromPreferences()
            var refs: Set<Data> = []
            var tunnelNames: Set<String> = []
            for (index, tunnelManager) in tunnelManagers.enumerated().reversed() {
                if let tunnelName = tunnelManager.localizedDescription {
                    tunnelNames.insert(tunnelName)
                }
                guard let proto = tunnelManager.protocolConfiguration as? NETunnelProviderProtocol else { continue }
#if os(iOS)
                let passwordRef = proto.verifyConfigurationReference() ? proto.passwordReference : nil
#elseif os(macOS)
                let passwordRef: Data?
                if proto.providerConfiguration?["UID"] as? uid_t == getuid() {
                    passwordRef = proto.verifyConfigurationReference() ? proto.passwordReference : nil
                } else {
                    passwordRef = proto.passwordReference // To handle multiple users in macOS, we skip verifying
                }
#else
#error("Unimplemented")
#endif
                if let ref = passwordRef {
                    refs.insert(ref)
                } else {
                    tunnelManager.removeFromPreferences { _ in }
                    tunnelManagers.remove(at: index)
                }
            }
            Keychain.deleteReferences(except: refs)
            let tunnels = tunnelManagers.map {
                Tunnel(tunnel: $0)
            }
            return tunnels
        } catch {
            throw TunnelsManagerError.tunnelList(error: error)
        }
    }
}

// MARK: - Observation -
private extension TunnelsManager {
    func observeTunnelStatuses() {
        NotificationCenter.default.publisher(for: .NEVPNStatusDidChange)
            .sink { [weak self] statusChangeNotification in
                guard
                    let self,
                    let session = statusChangeNotification.object as? NETunnelProviderSession,
                    let tunnelProvider = session.manager as? NETunnelProviderManager,
                    let tunnel = self.tunnels.first(where: { $0.tunnel == tunnelProvider })
                else {
                    return
                }
                logger.log(
                    level: .debug,
                    "Tunnel '\(tunnel.name)' connection status changed to '\(tunnel.tunnel.connection.status)'"
                )

                if tunnel.status == .restarting && session.status == .disconnected {
                    tunnel.connect()
                    return
                }
                tunnel.updateStatus()
            }
            .store(in: &cancellables)
    }
}
