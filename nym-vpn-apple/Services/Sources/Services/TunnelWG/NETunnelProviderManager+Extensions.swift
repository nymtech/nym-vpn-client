import NetworkExtension
import WireGuardKit

extension NETunnelProviderManager {
    private static var cachedConfigKey: UInt8 = 0

    public func setTunnelConfiguration(_ tunnelConfiguration: TunnelConfiguration) {
        protocolConfiguration = NETunnelProviderProtocol(
            tunnelConfiguration: tunnelConfiguration,
            previouslyFrom: protocolConfiguration
        )
        localizedDescription = tunnelConfiguration.name
        objc_setAssociatedObject(
            self,
            &NETunnelProviderManager.cachedConfigKey,
            tunnelConfiguration,
            objc_AssociationPolicy.OBJC_ASSOCIATION_RETAIN_NONATOMIC
        )
    }
}
