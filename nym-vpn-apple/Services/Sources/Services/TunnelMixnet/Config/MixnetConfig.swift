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
    let logPath: String
    let customDns: [IpAddr]
#endif
    public let entryGateway: EntryGateway
    public let exitRouter: ExitRouter
    public let isTwoHopEnabled: Bool
    public let bridgeMode: BridgeModeSetting
    public let isStealthApiEnabled: Bool
    public let isAdBlockingEnabled: Bool
    public let isErrorReportingEnabled: Bool
    public let isStatisticsEnabled: Bool
    public let isLanBypassEnabled: Bool
    public let mixnetTuning: MixnetTuningConfig
#if os(iOS)
    public let gatewaySelectionAlgorithmConfig: NymGatewaySelectionAlgorithmConfig
    public let isServerFamilyRemindersEnabled: Bool
#endif

    public var name = "NymVPN"
#if os(iOS)
    public init(
        entryGateway: EntryGateway,
        exitRouter: ExitRouter,
        configPath: String,
        dataPath: String,
        logPath: String,
        customDns: [IpAddr],
        mixnetTuning: MixnetTuningConfig,
        isErrorReportingEnabled: Bool,
        isStatisticsEnabled: Bool,
        bridgeMode: BridgeModeSetting,
        isStealthApiEnabled: Bool,
        isLanBypassEnabled: Bool,
        isAdBlockingEnabled: Bool,
        isTwoHopEnabled: Bool = false,
        gatewaySelectionAlgorithmConfig: NymGatewaySelectionAlgorithmConfig = NymGatewaySelectionAlgorithmConfig(),
        isServerFamilyRemindersEnabled: Bool = true,
        name: String = "NymVPN"
    ) {
        self.entryGateway = entryGateway
        self.exitRouter = exitRouter
        self.configPath = configPath
        self.dataPath = dataPath
        self.logPath = logPath
        self.customDns = customDns
        self.mixnetTuning = mixnetTuning
        self.isErrorReportingEnabled = isErrorReportingEnabled
        self.isStatisticsEnabled = isStatisticsEnabled
        self.bridgeMode = bridgeMode
        self.isStealthApiEnabled = isStealthApiEnabled
        self.isLanBypassEnabled = isLanBypassEnabled
        self.isTwoHopEnabled = isTwoHopEnabled
        self.isAdBlockingEnabled = isAdBlockingEnabled
        self.gatewaySelectionAlgorithmConfig = gatewaySelectionAlgorithmConfig
        self.isServerFamilyRemindersEnabled = isServerFamilyRemindersEnabled
        self.name = name
    }
#endif

    private enum CodingKeys: String, CodingKey {
        case entryGateway, exitRouter, isTwoHopEnabled, bridgeMode, isStealthApiEnabled
        case isAdBlockingEnabled, isErrorReportingEnabled, isStatisticsEnabled, isLanBypassEnabled
        case mixnetTuning, name
#if os(iOS)
        case configPath, dataPath, logPath, customDns, gatewaySelectionAlgorithmConfig, isServerFamilyRemindersEnabled
#endif
    }

    private enum LegacyCodingKeys: String, CodingKey {
        case isQuicEnabled
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        entryGateway = try container.decode(EntryGateway.self, forKey: .entryGateway)
        exitRouter = try container.decode(ExitRouter.self, forKey: .exitRouter)
        isTwoHopEnabled = try container.decode(Bool.self, forKey: .isTwoHopEnabled)
        // Stored before the bridge mode existed: carry the old QUIC flag over so the tunnel can still start.
        if let mode = try container.decodeIfPresent(BridgeModeSetting.self, forKey: .bridgeMode) {
            bridgeMode = mode
        } else {
            let legacy = try decoder.container(keyedBy: LegacyCodingKeys.self)
            bridgeMode = try legacy.decode(Bool.self, forKey: .isQuicEnabled) ? .on : .off
        }
        isStealthApiEnabled = try container.decode(Bool.self, forKey: .isStealthApiEnabled)
        isAdBlockingEnabled = try container.decode(Bool.self, forKey: .isAdBlockingEnabled)
        isErrorReportingEnabled = try container.decode(Bool.self, forKey: .isErrorReportingEnabled)
        isStatisticsEnabled = try container.decode(Bool.self, forKey: .isStatisticsEnabled)
        isLanBypassEnabled = try container.decode(Bool.self, forKey: .isLanBypassEnabled)
        mixnetTuning = try container.decode(MixnetTuningConfig.self, forKey: .mixnetTuning)
        name = try container.decode(String.self, forKey: .name)
#if os(iOS)
        configPath = try container.decode(String.self, forKey: .configPath)
        dataPath = try container.decode(String.self, forKey: .dataPath)
        logPath = try container.decode(String.self, forKey: .logPath)
        customDns = try container.decode([IpAddr].self, forKey: .customDns)
        gatewaySelectionAlgorithmConfig = try container.decode(
            NymGatewaySelectionAlgorithmConfig.self,
            forKey: .gatewaySelectionAlgorithmConfig
        )
        isServerFamilyRemindersEnabled = try container.decode(Bool.self, forKey: .isServerFamilyRemindersEnabled)
#endif
    }
}

#if os(iOS)
// MARK: - VpnConfig -
extension MixnetConfig {
    public func asVpnConfig(tunProvider: OsTunProvider) throws -> VpnConfig {
        VpnConfig(
            configDir: configPath,
            dataDir: dataPath,
            logDir: logPath,
            entryGateway: entryGateway.entryPoint,
            exitRouter: exitRouter.exitPoint,
            enableTwoHop: isTwoHopEnabled,
            bridgeMode: bridgeMode.bridgeMode,
            residentialExit: false,
            enableAdBlocking: isAdBlockingEnabled,
            frontingMode: isStealthApiEnabled ? .always : .onRetry,
            customDns: CustomDnsSettings(plainDnsAddresses: customDns),
            mixnetTraffic: mixnetTuning.mixnetTrafficConfig(),
            networkStats: NetworkStatisticsConfig(enabled: isStatisticsEnabled, allowDisconnected: false),
            gatewaySelectionAlgorithmConfig: gatewaySelectionAlgorithmConfig.sdkValue,
            gatewayIndependence: GatewayIndependence(
                enableNotifications: isServerFamilyRemindersEnabled,
                differentNodeFamily: true,
                differentAsn: true,
                differentSubnet: true
            ),
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
