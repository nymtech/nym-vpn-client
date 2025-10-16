import NetworkExtension

@MainActor extension NETunnelProviderManager {
    private static var cachedConfigKey: UInt8 = 0

    @MainActor public func setTunnelConfiguration(_ mixnetConfiguration: MixnetConfig) {
        guard let proto = NETunnelProviderProtocol(mixnetConfiguration: mixnetConfiguration) else { return }
        protocolConfiguration = proto
        localizedDescription = mixnetConfiguration.name
        objc_setAssociatedObject(
            self,
            &NETunnelProviderManager.cachedConfigKey,
            mixnetConfiguration,
            .OBJC_ASSOCIATION_RETAIN_NONATOMIC
        )
    }
}
