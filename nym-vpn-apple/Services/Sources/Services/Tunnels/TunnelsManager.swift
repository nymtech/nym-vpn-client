import Combine
import NetworkExtension
import Logging
import Constants
import ErrorReason
import Keychain
#if os(iOS)
import ErrorHandler
#endif

public final class TunnelsManager: ObservableObject {
    private var cancellables = Set<AnyCancellable>()

    public static let shared = TunnelsManager()

    @Published public var isLoaded: Result<Void, Error>?
    @Published public var activeTunnel: Tunnel?
    @Published public var lastError: Error?
    public var tunnels = [Tunnel]()
    public var logger = Logger(label: "TunnelsManager")

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

    public func resetVpnProfile() {
        Task {
            do {
                var tunnelManagers = try await NETunnelProviderManager.loadAllFromPreferences()
                for (index, tunnelManager) in tunnelManagers.enumerated().reversed() {
                    tunnelManager.removeFromPreferences { [weak self] error in
                        if let error = error {
                            self?.logger.error("Failed to remove VPN profile: \(error.localizedDescription)")
                        } else {
                            self?.logger.info("VPN profile removed successfully.")
                        }
                    }
                    tunnelManagers.remove(at: index)
                }
                Keychain.deleteReferences(except: [])
                try await loadTunnels()
            } catch {
                logger.error("Failed to reset VPN profile: \(error.localizedDescription)")
            }
        }
    }
}

// MARK: - Connection -
extension TunnelsManager {
    public func connect(tunnel: Tunnel) async throws {
        guard tunnels.contains(tunnel), tunnel.status == .disconnected  else { return }
#if targetEnvironment(simulator)
        tunnel.status = .connected
#else
        activeTunnel = tunnel
        do {
            try await tunnel.connect()
        } catch {
            throw error
        }
#endif
    }

    public func disconnect(tunnel: Tunnel) {
        guard tunnel.status != .disconnected && tunnel.status != .disconnecting else { return }
#if targetEnvironment(simulator)
        tunnel.status = .disconnected
#else
        tunnel.disconnect()
#endif
    }
}

// MARK: - Load All Tunnel Managers -
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
#if os(iOS)
                Task { [weak self] in
                    await self?.updateLastTunnelErrorIfNeeded()
                }
#endif
                tunnel.updateStatus()
            }
            .store(in: &cancellables)
    }

#if os(iOS)
    func updateLastTunnelErrorIfNeeded() async {
        guard activeTunnel?.status == .disconnecting else { return }

        do {
            try await activeTunnel?.tunnel.connection.fetchLastDisconnectError()
        } catch let error as NSError {
            switch error.domain {
            case VPNErrorReason.domain:
                lastError = VPNErrorReason(nsError: error)
            case ErrorReason.domain:
                lastError = ErrorReason(nsError: error)
            default:
                lastError = GeneralNymError.somethingWentWrong
            }
        } catch {
            lastError = GeneralNymError.somethingWentWrong
        }
    }
#endif
}
