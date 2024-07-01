import Foundation
import Network
import Constants
import CountriesManager
import CredentialsManager
#if os(iOS)
import MixnetLibrary
#endif

public struct MixnetConfig: Codable, Equatable {
#if os(iOS)
    let apiUrlString: String
    let explorerURLString: String
    let credentialsDataPath: String
#endif
    public let entryGateway: EntryGateway?
    public let exitRouter: ExitRouter
    public let isTwoHopEnabled: Bool

    public var name = "NymVPN Mixnet"
#if os(iOS)
    public init(
        entryGateway: EntryGateway,
        exitRouter: ExitRouter,
        credentialsDataPath: String,
        isTwoHopEnabled: Bool = false,
        name: String = "NymVPN Mixnet",
        apiUrlString: String = Constants.apiUrl.rawValue,
        explorerURLString: String = Constants.explorerURL.rawValue
    ) {
        self.entryGateway = entryGateway
        self.exitRouter = exitRouter
        self.credentialsDataPath = credentialsDataPath
        self.isTwoHopEnabled = isTwoHopEnabled
        self.name = name
        self.apiUrlString = apiUrlString
        self.explorerURLString = explorerURLString
    }
#endif

#if os(macOS)
    public init(
        entryGateway: EntryGateway,
        exitRouter: ExitRouter,
        isTwoHopEnabled: Bool = false
    ) {
        self.entryGateway = entryGateway
        self.exitRouter = exitRouter
        self.isTwoHopEnabled = isTwoHopEnabled
    }
#endif
}

#if os(iOS)
// MARK: - VpnConfig -
extension MixnetConfig {
    public func asVpnConfig(mixnetTunnelProvider: MixnetTunnelProvider) throws -> VpnConfig {
        guard
            let apiURL = URL(string: Constants.apiUrl.rawValue),
            let explorerURL = URL(string: Constants.explorerURL.rawValue)
        else {
            throw GeneralNymError.invalidUrl
        }
        return VpnConfig(
            apiUrl: apiURL,
            explorerUrl: explorerURL,
            entryGateway: entryGateway?.entryPoint ?? .randomLowLatency,
            exitRouter: exitRouter.exitPoint,
            enableTwoHop: isTwoHopEnabled,
            tunProvider: mixnetTunnelProvider,
            credentialDataPath: credentialsDataPath
        )
    }
}
#endif

// MARK: - JSON -
extension MixnetConfig {
    // TODO: inject JSONEncoder + JSONDecoder
    public func toJson() -> String? {
        let encoder = JSONEncoder()
        guard let jsonData = try? encoder.encode(self) else { return nil }
        return String(data: jsonData, encoding: .utf8)
    }

    public static func from(jsonString: String) -> MixnetConfig? {
        let decoder = JSONDecoder()
        guard let jsonData = jsonString.data(using: .utf8) else { return nil }
        return try? decoder.decode(MixnetConfig.self, from: jsonData)
    }
}
