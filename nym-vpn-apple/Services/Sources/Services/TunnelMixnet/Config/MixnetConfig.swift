import Foundation
import Network
import Constants
import ConnectionTypes
import CountriesManager
import CredentialsManager
#if os(iOS)
import AppVersionProvider
import ConfigurationManager
import MixnetLibrary
#endif

public struct MixnetConfig: Codable, Equatable {
#if os(iOS)
    let credentialsDataPath: String
#endif
    public let entryGateway: EntryGateway
    public let exitRouter: ExitRouter
    public let isTwoHopEnabled: Bool

    public var name = "NymVPN Mixnet"
#if os(iOS)
    public init(
        entryGateway: EntryGateway,
        exitRouter: ExitRouter,
        credentialsDataPath: String,
        isTwoHopEnabled: Bool = false,
        name: String = "NymVPN Mixnet"
    ) {
        self.entryGateway = entryGateway
        self.exitRouter = exitRouter
        self.credentialsDataPath = credentialsDataPath
        self.isTwoHopEnabled = isTwoHopEnabled
        self.name = name
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
    public func asVpnConfig(tunProvider: OsTunProvider, tunStatusListener: TunnelStatusListener?) throws -> VpnConfig {
        VpnConfig(
            entryGateway: entryGateway.entryPoint,
            exitRouter: exitRouter.exitPoint,
            enableTwoHop: isTwoHopEnabled,
            tunProvider: tunProvider,
            credentialDataPath: credentialsDataPath,
            tunStatusListener: tunStatusListener,
            credentialMode: nil,
            statisticsRecipient: nil,
            userAgent: UserAgent(
                application: AppVersionProvider.app,
                version: "\(AppVersionProvider.appVersion()) (\(AppVersionProvider.libVersion))",
                platform: AppVersionProvider.platform,
                gitCommit: ""
            )
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
