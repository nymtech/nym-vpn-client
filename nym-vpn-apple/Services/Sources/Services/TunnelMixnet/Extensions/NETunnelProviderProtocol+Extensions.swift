import NetworkExtension
import Keychain

extension NETunnelProviderProtocol {
    convenience init?(mixnetConfiguration: MixnetConfig) {
        self.init()
        guard
            let appId = Bundle.main.bundleIdentifier,
            let configEncoded = try? JSONEncoder().encode(mixnetConfiguration),
            let configString = String(data: configEncoded, encoding: .utf8)
        else {
            return nil
        }

        providerBundleIdentifier = "\(appId).network-extension"
        passwordReference = Keychain.makeReference(containing: configString, called: mixnetConfiguration.name)
        guard passwordReference != nil else { return nil }
        // TODO: Mixnet - What server address should we be using?
        serverAddress = "Unspecified"
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
            let configData = encodedConfig.data(using: .utf8),
            let mixnetConfig = try? JSONDecoder().decode(MixnetConfig.self, from: configData)
        else {
            return nil
        }
        return mixnetConfig
    }
}
