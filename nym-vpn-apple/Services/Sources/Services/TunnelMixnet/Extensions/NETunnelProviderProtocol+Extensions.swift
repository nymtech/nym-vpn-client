import NetworkExtension
import Keychain

extension NETunnelProviderProtocol {
    convenience init?(mixnetConfiguration: MixnetConfig) {
        self.init()
        guard
            let appId = Bundle.main.bundleIdentifier,
            let configString = mixnetConfiguration.toJson()
        else {
            return nil
        }

        providerBundleIdentifier = "\(appId).network-extension"
        serverAddress = "127.0.0.1"
        passwordReference = Keychain.makeReference(containing: configString, called: mixnetConfiguration.name)

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
            let encodedConfig = Keychain.openReference(called: passwordReference),
            let mixnetConfig = MixnetConfig.from(jsonString: encodedConfig)
        else {
            return nil
        }
        return mixnetConfig
    }
}
