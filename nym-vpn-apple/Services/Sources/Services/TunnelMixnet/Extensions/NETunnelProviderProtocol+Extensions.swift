import NetworkExtension

@MainActor extension NETunnelProviderProtocol {
    convenience init?(mixnetConfiguration: MixnetConfig) {
        self.init()

        guard let appId = Bundle.main.bundleIdentifier else { return nil }
        guard MixnetConfigStorage.save(mixnetConfiguration) else { return nil }

        providerBundleIdentifier = "\(appId).network-extension"
        serverAddress = "127.0.0.1"
    }

    public func destroyConfigurationReference() {
        MixnetConfigStorage.delete()
    }

    public func asMixnetConfig(called name: String? = nil) -> MixnetConfig? {
        MixnetConfigStorage.load()
    }
}
