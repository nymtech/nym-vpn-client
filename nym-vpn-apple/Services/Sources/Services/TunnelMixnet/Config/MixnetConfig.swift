import Foundation
import Network
import AppSettings
import Constants
import ConnectionTypes
import CountriesManager
import CredentialsManager
#if os(iOS)
import AppVersionProvider
import ConfigurationManager
import NymVPNLib
#endif

public struct MixnetConfig: Codable, Equatable {
#if os(iOS)
    let credentialsDataPath: String
    let configPath: String
#endif
    public let entryGateway: EntryGateway
    public let exitRouter: ExitRouter
    public let isTwoHopEnabled: Bool
    public let isErrorReportingEnabled: Bool
    public let isStatisticsEnabled: Bool

    public var name = "NymVPN Mixnet"
#if os(iOS)
    public init(
        entryGateway: EntryGateway,
        exitRouter: ExitRouter,
        credentialsDataPath: String,
        configPath: String,
        isErrorReportingEnabled: Bool,
        isStatisticsEnabled: Bool,
        isTwoHopEnabled: Bool = false,
        name: String = "NymVPN Mixnet"
    ) {
        self.entryGateway = entryGateway
        self.exitRouter = exitRouter
        self.credentialsDataPath = credentialsDataPath
        self.configPath = configPath
        self.isErrorReportingEnabled = isErrorReportingEnabled
        self.isStatisticsEnabled = isStatisticsEnabled
        self.isTwoHopEnabled = isTwoHopEnabled
        self.name = name
    }
#endif

#if os(macOS)
    public init(
        entryGateway: EntryGateway,
        exitRouter: ExitRouter,
        isErrorReportingEnabled: Bool,
        isStatisticsEnabled: Bool,
        isTwoHopEnabled: Bool = false
    ) {
        self.entryGateway = entryGateway
        self.exitRouter = exitRouter
        self.isErrorReportingEnabled = isErrorReportingEnabled
        self.isStatisticsEnabled = isStatisticsEnabled
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
            enableBridges: false,
            residentialExit: false,
            tunProvider: tunProvider,
            configPath: configPath,
            credentialDataPath: credentialsDataPath,
            tunStatusListener: tunStatusListener,
            statisticsRecipient: nil,
            userAgent: .appUserAgent
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
