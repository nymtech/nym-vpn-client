import NetworkExtension
import Tunnels

extension TunnelsManager {
    public func addUpdate(
        tunnelConfiguration: MixnetConfig,
        isOndemandEnabled: Bool
    ) async throws -> Tunnel {
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

        let alwaysOnRule = NEOnDemandRuleConnect()
        alwaysOnRule.interfaceTypeMatch = .any
        tunnelProviderManager.onDemandRules = [alwaysOnRule]
        tunnelProviderManager.isOnDemandEnabled = isOndemandEnabled

        let activeTunnel = tunnels.first { $0.status == .connected || $0.status == .connecting }

        do {
            try await tunnelProviderManager.saveToPreferences()
#if os(iOS)
            // HACK: In iOS, adding a tunnel causes deactivation of any currently active tunnel.
            // This is an ugly hack to reactivate the tunnel that has been deactivated like that.
            if let activeTunnel = activeTunnel {
                if activeTunnel.status == .connected || activeTunnel.status == .connecting {
                    try await connect(tunnel: activeTunnel)
                }
                if activeTunnel.status == .connected || activeTunnel.status == .connecting {
                    activeTunnel.status = .restarting
                }
            }
#endif
            if !self.tunnels.contains(where: { $0.name == tunnelConfiguration.name }) {
                self.tunnels.append(tunnel)
            }
            return tunnel
        } catch {
            logger.log(level: .error, "Saving configuration failed: \(error)")
            let protocolConfiguration = tunnelProviderManager.protocolConfiguration as? NETunnelProviderProtocol
            protocolConfiguration?.destroyConfigurationReference()
            throw TunnelsManagerError.addTunnel(error: error)
        }
    }
}
