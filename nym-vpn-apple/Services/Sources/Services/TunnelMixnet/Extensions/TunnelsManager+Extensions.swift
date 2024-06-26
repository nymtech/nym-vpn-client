import NetworkExtension
import Tunnels

extension TunnelsManager {
    public func add(
        tunnelConfiguration: MixnetConfig,
        onDemandOption: OnDemandRule = .off,
        completionHandler: @escaping (Result<Tunnel, TunnelsManagerError>) -> Void
    ) {
        let tunnelProviderManager: NETunnelProviderManager
        let tunnel: Tunnel
        if let existingTunnel = tunnels.first(where: { $0.name == tunnelConfiguration.name }) {
            tunnelProviderManager = existingTunnel.tunnel
            tunnel = existingTunnel
        } else {
            tunnelProviderManager = NETunnelProviderManager()
            tunnel = Tunnel(tunnel: tunnelProviderManager)
        }

        tunnelProviderManager.setTunnelConfiguration(tunnelConfiguration)
        tunnelProviderManager.isEnabled = true
        // TODO: add on demand rules support
        // onDemandOption.apply(on: tunnelProviderManager)

        let activeTunnel = tunnels.first { $0.status == .connected || $0.status == .connecting }
        tunnelProviderManager.saveToPreferences { [weak self] error in
            if let error = error {
                self?.logger.log(level: .error, "Saving configuration failed: \(error)")
                let protocolConfiguration = (tunnelProviderManager.protocolConfiguration as? NETunnelProviderProtocol)
                protocolConfiguration?.destroyConfigurationReference()
                completionHandler(.failure(TunnelsManagerError.addTunnel(error: error)))
                return
            }

            guard let self = self else { return }

            #if os(iOS)
            // HACK: In iOS, adding a tunnel causes deactivation of any currently active tunnel.
            // This is an ugly hack to reactivate the tunnel that has been deactivated like that.
            if let activeTunnel = activeTunnel {
                if activeTunnel.status == .connected || activeTunnel.status == .connecting {
                    self.connect(tunnel: activeTunnel)
                }
                if activeTunnel.status == .connected || activeTunnel.status == .connecting {
                    activeTunnel.status = .restarting
                }
            }
            #endif

            if !self.tunnels.contains(where: { $0.name == tunnelConfiguration.name }) {
                self.tunnels.append(tunnel)
            }

            // self.tunnels.sort { TunnelsManager.tunnelNameIsLessThan($0.name, $1.name) }
            completionHandler(.success(tunnel))
        }
    }
}
