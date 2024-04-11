import NetworkExtension

extension NETunnelProviderManager {
    private static var cachedConfigKey: UInt8 = 0

    func setTunnelConfiguration(_ mixnetConfiguration: MixnetConfig) {
        protocolConfiguration = NETunnelProviderProtocol(mixnetConfiguration: mixnetConfiguration)
        localizedDescription = mixnetConfiguration.name
        objc_setAssociatedObject(
            self,
            &NETunnelProviderManager.cachedConfigKey,
            mixnetConfiguration,
            objc_AssociationPolicy.OBJC_ASSOCIATION_RETAIN_NONATOMIC
        )
    }
}
