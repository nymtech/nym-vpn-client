import NetworkExtension
import Keychain

@MainActor extension NETunnelProviderProtocol {
    convenience init?(mixnetConfiguration: MixnetConfig) {
        self.init()

        guard
            let appId = Bundle.main.bundleIdentifier,
            let configString = mixnetConfiguration.toJson()
        else { return nil }

        providerBundleIdentifier = "\(appId).network-extension"
        serverAddress = "127.0.0.1"

        passwordReference = Keychain.updateReferenceOrCreateNew(
            called: mixnetConfiguration.name,
            with: configString
        )

        if passwordReference == nil {
            return nil
        }
    }

    public func destroyConfigurationReference() {
        guard let ref = passwordReference else { return }
        Keychain.deleteReference(called: ref)
    }

    public func verifyConfigurationReference() -> Bool {
        guard let ref = passwordReference else { return false }
        return Keychain.verifyReference(called: ref)
    }

    public func asMixnetConfig(called name: String? = nil) -> MixnetConfig? {
        guard
            let passwordReference,
            let encoded = Keychain.openReference(called: passwordReference),
            let cfg = MixnetConfig.from(jsonString: encoded)
        else { return nil }
        return cfg
    }
}
