import NetworkExtension
import WireGuardKit
import Keychain

extension NETunnelProviderProtocol {
    convenience init?(tunnelConfiguration: TunnelConfiguration, previouslyFrom old: NEVPNProtocol? = nil) {
        self.init()

        guard
            let name = tunnelConfiguration.name,
            let appId = Bundle.main.bundleIdentifier
        else {
            return nil
        }

        providerBundleIdentifier = "\(appId).network-extension"
        passwordReference = Keychain.makeReference(
            containing: tunnelConfiguration.asWgQuickConfig(),
            called: name,
            previouslyReferencedBy: old?.passwordReference
        )

        if passwordReference == nil {
            return nil
        }
        #if os(macOS)
            providerConfiguration = ["UID": getuid()]
        #endif

        let endpoints = tunnelConfiguration.peers.compactMap { $0.endpoint }
        if endpoints.count == 1 {
            serverAddress = endpoints[0].stringRepresentation
        } else if endpoints.isEmpty {
            serverAddress = "Unspecified"
        } else {
            serverAddress = "Multiple endpoints"
        }
    }

    public func asTunnelConfiguration(called name: String? = nil) -> TunnelConfiguration? {
        if let passwordReference = passwordReference, let config = Keychain.openReference(called: passwordReference) {
            return try? TunnelConfiguration(fromWgQuickConfig: config, called: name)
        }
        return nil
    }
}
