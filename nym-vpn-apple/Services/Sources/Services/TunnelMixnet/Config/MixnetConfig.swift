import Foundation
import Network
import AppSettings
import Constants
import ConnectionTypes
import CredentialsManager
#if os(iOS)
import AppVersionProvider
import ConfigurationManager
import NymVPNLib
#endif

public struct MixnetConfig: Codable, Equatable {
#if os(iOS)
    let configPath: String
    let dataPath: String
    let customDns: [IpAddr]
#endif
    public let entryGateway: EntryGateway
    public let exitRouter: ExitRouter
    public let isTwoHopEnabled: Bool
    public let isQuicEnabled: Bool
    public let isLewesEnabled: Bool
    public let isErrorReportingEnabled: Bool
    public let isStatisticsEnabled: Bool
    public let isLanBypassEnabled: Bool
    public let mixnetTuning: MixnetTuningConfig

    public var name = "NymVPN Mixnet"
#if os(iOS)
    public init(
        entryGateway: EntryGateway,
        exitRouter: ExitRouter,
        configPath: String,
        dataPath: String,
        customDns: [IpAddr],
        mixnetTuning: MixnetTuningConfig,
        isErrorReportingEnabled: Bool,
        isStatisticsEnabled: Bool,
        isQuicEnabled: Bool,
        isLanBypassEnabled: Bool,
        isLewesEnabled: Bool,
        isTwoHopEnabled: Bool = false,
        name: String = "NymVPN Mixnet"
    ) {
        self.entryGateway = entryGateway
        self.exitRouter = exitRouter
        self.configPath = configPath
        self.dataPath = dataPath
        self.customDns = customDns
        self.mixnetTuning = mixnetTuning
        self.isErrorReportingEnabled = isErrorReportingEnabled
        self.isStatisticsEnabled = isStatisticsEnabled
        self.isQuicEnabled = isQuicEnabled
        self.isLanBypassEnabled = isLanBypassEnabled
        self.isTwoHopEnabled = isTwoHopEnabled
        self.isLewesEnabled = isLewesEnabled
        self.name = name
    }
#endif
}

#if os(iOS)
// MARK: - VpnConfig -
extension MixnetConfig {
    public func asVpnConfig(tunProvider: OsTunProvider) throws -> VpnConfig {
        VpnConfig(
            configDir: configPath,
            dataDir: dataPath,
            entryGateway: entryGateway.entryPoint,
            exitRouter: exitRouter.exitPoint,
            enableTwoHop: isTwoHopEnabled,
            enableBridges: isQuicEnabled,
            enableLewesProtocol: isLewesEnabled,
            residentialExit: false,
            customDns: customDns,
            mixnetTraffic: mixnetTuning.mixnetTrafficConfig(),
            networkStats: nil,
            userAgent: .appUserAgent,
            tunProvider: tunProvider
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
