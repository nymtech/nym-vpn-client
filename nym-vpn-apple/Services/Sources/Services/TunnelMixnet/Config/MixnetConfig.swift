import Foundation
import Network
import Constants
import CountriesManager
import CredentialsManager
#if os(iOS)
import MixnetLibrary
#endif

public struct MixnetConfig: Codable, Equatable {
    let apiUrlString: String
    let explorerURLString: String
    public let entryGateway: EntryGateway?
    public let exitRouter: ExitRouter
    public let isTwoHopEnabled: Bool
    let credentialsDataPath: String

    public var name = "NymVPN Mixnet"

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
