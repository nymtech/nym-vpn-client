#if os(iOS)
import NetworkExtension

public enum NymTunnelManager {
    private static let providerBundleID = "net.nymtech.vpn.network-extension"

    public static func loadManager() async throws -> NETunnelProviderManager? {
        let managers = try await NETunnelProviderManager.loadAllFromPreferences()
        return managers.first(
            where: {
                ($0.protocolConfiguration as? NETunnelProviderProtocol)?
                    .providerBundleIdentifier == providerBundleID
            }
        )
    }
}
#endif
