import Combine
import NetworkExtension
import Logging
import Constants
import ErrorReason
import Keychain
#if os(iOS)
import ErrorHandler
#endif

@MainActor public final class TunnelsManager: ObservableObject {
    private var cancellables = Set<AnyCancellable>()

    public static let shared = TunnelsManager()

    @Published public var isLoaded: Result<Void, Error>?
    @Published public var activeTunnel: Tunnel?
    @Published public var lastError: Error?
    public var tunnels = [Tunnel]()
    public var logger = Logger(label: "TunnelsManager")

    init() {
        Task { [weak self] in
            guard let self else { return }
            try? await self.loadTunnels()
            self.observeTunnelStatuses()
        }
    }
}

// MARK: - Management
extension TunnelsManager {
    public func loadTunnels() async throws {
        do {
            let loaded = try await loadAllTunnelManagers()
            activeTunnel = loaded.first { $0.tunnel.isEnabled }
            tunnels = loaded
            isLoaded = .success(())
        } catch {
            logger.log(level: .error, "Failed loading tunnel managers with \(error)")
            isLoaded = .failure(error)
            throw error
        }
    }

    public func resetVpnProfile() {
        Task { [weak self] in
            guard let self else { return }
            do {
                var managers = try await NETunnelProviderManager.loadAllFromPreferences()
                for (idx, manager) in managers.enumerated().reversed() {
                    manager.removeFromPreferences { [weak self] error in
                        Task { @MainActor [weak self] in
                            if let error {
                                self?.logger.error("Failed to remove VPN profile: \(error.localizedDescription)")
                            } else {
                                self?.logger.info("VPN profile removed successfully.")
                            }
                        }
                    }
                    managers.remove(at: idx)
                }
                Keychain.deleteReferences(except: [])
                try await loadTunnels()
            } catch {
                logger.error("Failed to reset VPN profile: \(error.localizedDescription)")
            }
        }
    }
}

// MARK: - Connection
extension TunnelsManager {
    public func connect(tunnel: Tunnel) async throws {
        guard tunnels.contains(tunnel) else { return }
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

// MARK: - Load All Tunnel Managers
private extension TunnelsManager {
    func loadAllTunnelManagers() async throws -> [Tunnel] {
        do {
            var managers = try await NETunnelProviderManager.loadAllFromPreferences()
            var refs: Set<Data> = []
            var tunnelNames: Set<String> = []

            for (idx, manager) in managers.enumerated().reversed() {
                if let name = manager.localizedDescription { tunnelNames.insert(name) }

                guard let proto = manager.protocolConfiguration as? NETunnelProviderProtocol else { continue }

#if os(iOS)
                let passwordRef = proto.verifyConfigurationReference() ? proto.passwordReference : nil
#elseif os(macOS)
                let passwordRef: Data?
                if proto.providerConfiguration?["UID"] as? uid_t == getuid() {
                    passwordRef = proto.verifyConfigurationReference() ? proto.passwordReference : nil
                } else {
                    // Multiple users on macOS: skip verification to avoid false negatives
                    passwordRef = proto.passwordReference
                }
#else
#error("Unimplemented")
#endif

                if let ref = passwordRef {
                    refs.insert(ref)
                } else {
                    manager.removeFromPreferences { _ in }
                    managers.remove(at: idx)
                }
            }

            Keychain.deleteReferences(except: refs)

            return managers.map { Tunnel(tunnel: $0) }
        } catch {
            throw TunnelsManagerError.tunnelList(error: error)
        }
    }
}

// MARK: - Observation
private extension TunnelsManager {
    func observeTunnelStatuses() {
        NotificationCenter.default.publisher(for: .NEVPNStatusDidChange)
            .sink { [weak self] note in
                guard
                    let self,
                    let session = note.object as? NETunnelProviderSession,
                    let provider = session.manager as? NETunnelProviderManager,
                    let tunnel = self.tunnels.first(where: { $0.tunnel == provider })
                else {
                    return
                }

                tunnel.updateStatus()
#if os(iOS)
                Task { [weak self] in
                    await self?.updateLastTunnelErrorIfNeeded()
                }
#endif
            }
            .store(in: &cancellables)
    }

#if os(iOS)
    func updateLastTunnelErrorIfNeeded() async {
        guard activeTunnel?.status == .disconnecting else { return }

        if let error = activeTunnel?.lastError as? NSError {
            switch error.domain {
            case VPNErrorReason.domain:
                lastError = VPNErrorReason(nsError: error)
            case ErrorReason.domain:
                lastError = ErrorReason(nsError: error)
            default:
                lastError = error
            }
        } else {
            lastError = nil
        }
    }
#endif
}
