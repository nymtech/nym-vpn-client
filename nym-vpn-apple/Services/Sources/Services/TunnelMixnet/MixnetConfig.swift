import Foundation
import Network
import MixnetLibrary

public struct MixnetConfig: Codable {
    public static let apiURL = "https://sandbox-nym-api1.nymtech.net/api"
    public static let explorerURL = "https://sandbox-explorer.nymtech.net/api"

    let apiUrl: String
    let explorerURL: String
    let entryGateway: EntryGateway
    let exitRouter: ExitRouter
    let isTwoHopEnabled: Bool

    public var name = "NymVPN Mixnet"

    public init(
        apiUrl: String = MixnetConfig.apiURL,
        explorerURL: String = MixnetConfig.explorerURL,
        entryGateway: EntryGateway = .randomLowLatency,
        exitRouter: ExitRouter = .randomLowLatency,
        isTwoHopEnabled: Bool = false,
        name: String = "NymVPN Mixnet"
    ) {
        self.apiUrl = apiUrl
        self.explorerURL = explorerURL
        self.entryGateway = entryGateway
        self.exitRouter = exitRouter
        self.isTwoHopEnabled = isTwoHopEnabled
        self.name = name
    }
}

// MARK: - VpnConfig -
extension MixnetConfig {
    public func asVpnConfig(mixnetTunnelProvider: MixnetTunnelProvider) -> VpnConfig {
        return VpnConfig(
            apiUrl: apiUrl,
            explorerUrl: explorerURL,
            entryGateway: .location(location: "GB"),
            exitRouter: .location(location: "GB"),
            enableTwoHop: false,
            tunProvider: mixnetTunnelProvider
        )
    }
}

// MARK: - Entry gateways -
extension MixnetConfig {
    public enum EntryGateway: Codable {
        case location(String)
        case randomLowLatency
    }
}

// MARK: - Exit router -
extension MixnetConfig {
    public enum ExitRouter: Codable {
        case location(String)
        case randomLowLatency
    }
}
