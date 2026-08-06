import Foundation
#if os(macOS)
import NymVPNLib
#endif

public struct ConnectionConfig: Codable {
    public var entry: EntryGateway
    public var exit: ExitRouter
    public var dns: [String]?
    public var allowLan: Bool
    public var disableIpv6: Bool
    public var enableTwoHop: Bool
    public var enableBridges: Bool
    public var enableAdBlocking: Bool
    public var netstack: Bool
    public var residentialExit: Bool
    public var mixnetTuningConfig: MixnetTuningConfig
    public var splitTunnelConfig: SplitTunnelConfig
    public var geoExclusionConfig: GeoExclusionConfig
    public var gatewaySelectionAlgorithmConfig: NymGatewaySelectionAlgorithmConfig

#if os(iOS)

#elseif os(macOS)
    public var entryPoint: EntryPoint {
        entryPoint(from: entry)
    }

    public var exitPoint: ExitPoint {
        exitPoint(from: exit)
    }
#endif

    public init(
        entry: EntryGateway,
        exit: ExitRouter,
        dns: [String]?,
        allowLan: Bool,
        disableIpv6: Bool,
        enableTwoHop: Bool,
        enableBridges: Bool,
        enableAdBlocking: Bool,
        netstack: Bool,
        residentialExit: Bool,
        mixnetTuningConfig: MixnetTuningConfig,
        splitTunnelConfig: SplitTunnelConfig,
        geoExclusionConfig: GeoExclusionConfig = GeoExclusionConfig(),
        gatewaySelectionAlgorithmConfig: NymGatewaySelectionAlgorithmConfig = NymGatewaySelectionAlgorithmConfig(enableGeoLocation: true)
    ) {
        self.entry = entry
        self.exit = exit
        self.dns = dns
        self.allowLan = allowLan
        self.disableIpv6 = disableIpv6
        self.enableTwoHop = enableTwoHop
        self.enableBridges = enableBridges
        self.enableAdBlocking = enableAdBlocking
        self.netstack = netstack
        self.residentialExit = residentialExit
        self.mixnetTuningConfig = mixnetTuningConfig
        self.splitTunnelConfig = splitTunnelConfig
        self.geoExclusionConfig = geoExclusionConfig
        self.gatewaySelectionAlgorithmConfig = gatewaySelectionAlgorithmConfig
    }

#if os(iOS)

#elseif os(macOS)
    public init(from config: VpnServiceConfig) {
        self.entry = ConnectionConfig.entryGateway(from: config.entryPoint)
        self.exit = ConnectionConfig.exitRouter(from: config.exitPoint)
        self.dns = config.customDns
        self.allowLan = config.allowLan
        self.disableIpv6 = config.disableIpv6
        self.enableTwoHop = config.enableTwoHop
        self.enableBridges = config.enableBridges
        self.netstack = config.netstack
        self.residentialExit = config.residentialExit
        self.mixnetTuningConfig = MixnetTuningConfig(from: config.mixnetTraffic)
        self.enableAdBlocking = config.enableAdBlocking
        self.splitTunnelConfig = SplitTunnelConfig(from: config.splitTunnel)
        self.geoExclusionConfig = GeoExclusionConfig(from: config.geoExclusion)
        self.gatewaySelectionAlgorithmConfig = NymGatewaySelectionAlgorithmConfig(
            enableGeoLocation: true
        )
    }
#endif
}

#if os(macOS)
private extension ConnectionConfig {
    static func entryGateway(from entryPoint: EntryPoint) -> EntryGateway {
        switch entryPoint {
        case let .gateway(identity):
            return .gateway(identity)
        case let .country(twoLetterIsoCountryCode):
            return .country(twoLetterIsoCountryCode)
        case let .region(region: region):
            return .region(countryCode: "", region: region)
        case .random:
            return .random
        case .auto:
            return .auto
        }
    }

    static func exitRouter(from exitPoint: ExitPoint) -> ExitRouter {
        switch exitPoint {
        case let .address(address):
            print("🔥 unused ExitPoint.address received from daemon (\(address)) — coercing to .random")
            return ExitRouter.random
        case let .gateway(identity):
            return ExitRouter.gateway(identity)
        case let .country(twoLetterIsoCountryCode):
            return ExitRouter.country(twoLetterIsoCountryCode)
        case let .region(region):
            return ExitRouter.region(countryCode: "", region: region)
        case .random:
            return ExitRouter.random
        case .auto:
            return ExitRouter.auto
        }
    }

    func entryPoint(from entryGateway: EntryGateway) -> EntryPoint {
        switch entryGateway {
        case let .country(code):
            EntryPoint.country(twoLetterIsoCountryCode: code)
        case let .gateway(node):
            EntryPoint.gateway(identity: node)
        case .random:
            EntryPoint.random
        case .auto:
            EntryPoint.auto(excludeUserCountry: true)
        case let .region(countryCode: _, region: region):
            EntryPoint.region(region: region)
        }
    }

    func exitPoint(from exitRouter: ExitRouter) -> ExitPoint {
        switch exitRouter {
        case let .country(code):
            ExitPoint.country(twoLetterIsoCountryCode: code)
        case let .gateway(node):
            ExitPoint.gateway(identity: node)
        case let .region(countryCode: _, region: region):
            ExitPoint.region(region: region)
        case .random:
            ExitPoint.random
        case .auto:
            ExitPoint.auto(excludeEntryPointCountry: true, excludeUserCountry: true)
        }
    }
}
#endif

extension ConnectionConfig: Equatable {
    public static func == (lhs: ConnectionConfig, rhs: ConnectionConfig) -> Bool {
        lhs.entry == rhs.entry &&
        lhs.exit == rhs.exit &&
        lhs.dns == rhs.dns &&
        lhs.allowLan == rhs.allowLan &&
        lhs.disableIpv6 == rhs.disableIpv6 &&
        lhs.enableTwoHop == rhs.enableTwoHop &&
        lhs.enableBridges == rhs.enableBridges &&
        lhs.netstack == rhs.netstack &&
        lhs.residentialExit == rhs.residentialExit &&
        lhs.mixnetTuningConfig == rhs.mixnetTuningConfig &&
        lhs.splitTunnelConfig == rhs.splitTunnelConfig &&
        lhs.geoExclusionConfig == rhs.geoExclusionConfig &&
        lhs.gatewaySelectionAlgorithmConfig == rhs.gatewaySelectionAlgorithmConfig
    }
}

extension ConnectionConfig {
    private enum CodingKeys: String, CodingKey {
        case entry, exit, dns, allowLan, disableIpv6, enableTwoHop
        case enableBridges, enableAdBlocking, netstack
        case residentialExit, mixnetTuningConfig, splitTunnelConfig
        case geoExclusionConfig
        case gatewaySelectionAlgorithmConfig
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        self.entry = try container.decode(EntryGateway.self, forKey: .entry)
        self.exit = try container.decode(ExitRouter.self, forKey: .exit)
        self.dns = try container.decodeIfPresent([String].self, forKey: .dns)
        self.allowLan = try container.decode(Bool.self, forKey: .allowLan)
        self.disableIpv6 = try container.decode(Bool.self, forKey: .disableIpv6)
        self.enableTwoHop = try container.decode(Bool.self, forKey: .enableTwoHop)
        self.enableBridges = try container.decode(Bool.self, forKey: .enableBridges)
        self.enableAdBlocking = try container.decode(Bool.self, forKey: .enableAdBlocking)
        self.netstack = try container.decode(Bool.self, forKey: .netstack)
        self.residentialExit = try container.decode(Bool.self, forKey: .residentialExit)
        self.mixnetTuningConfig = try container.decode(MixnetTuningConfig.self, forKey: .mixnetTuningConfig)
        self.splitTunnelConfig = try container.decode(SplitTunnelConfig.self, forKey: .splitTunnelConfig)
        self.geoExclusionConfig = try container.decodeIfPresent(
            GeoExclusionConfig.self,
            forKey: .geoExclusionConfig
        ) ?? GeoExclusionConfig()
        self.gatewaySelectionAlgorithmConfig = try container.decodeIfPresent(
            NymGatewaySelectionAlgorithmConfig.self,
            forKey: .gatewaySelectionAlgorithmConfig
        ) ?? NymGatewaySelectionAlgorithmConfig(enableGeoLocation: true)
    }
}

extension ConnectionConfig {
    public func toJson() -> String? {
        guard let jsonData = try? JSONEncoder().encode(self) else { return nil }
        return String(data: jsonData, encoding: .utf8)
    }

    public static func from(jsonString: String) -> ConnectionConfig? {
        guard let jsonData = jsonString.data(using: .utf8) else { return nil }
        return try? JSONDecoder().decode(ConnectionConfig.self, from: jsonData)
    }
}
