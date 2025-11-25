import NetworkExtension
import Tunnels

extension TunnelsManager {
    @MainActor public func addUpdate(
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

        tunnelProviderManager.protocolConfiguration?.excludeLocalNetworks = tunnelConfiguration.isLanBypassEnabled

        do {
            try await tunnelProviderManager.saveToPreferencesAndLoadTunnels()

            if !tunnels.contains(where: { $0.name == tunnelConfiguration.name }) {
                tunnels.append(tunnel)
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
