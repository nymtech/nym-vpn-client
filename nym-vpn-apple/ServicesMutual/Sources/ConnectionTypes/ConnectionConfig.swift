import Foundation
#if os(iOS)
#elseif os(macOS)
import NymVPNRpc
#endif

public final class ConnectionConfig: Codable {
    public var entry: EntryGateway
    public var exit: ExitRouter
    public var dns: [String]?
    public var allowLan: Bool
    public var disableIpv6: Bool
    public var enableTwoHop: Bool
    public var enableBridges: Bool
    public var enableLewes: Bool
    public var netstack: Bool
    public var minGatewayVpnPerformance: UInt8?
    public var residentialExit: Bool
    public var mixnetTuningConfig: MixnetTuningConfig

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
        dns: [String]? = nil,
        allowLan: Bool,
        disableIpv6: Bool,
        enableTwoHop: Bool,
        enableBridges: Bool,
        enableLewes: Bool,
        netstack: Bool,
        minGatewayVpnPerformance: UInt8? = nil,
        residentialExit: Bool,
        mixnetTuningConfig: MixnetTuningConfig
    ) {
        self.entry = entry
        self.exit = exit
        self.dns = dns
        self.allowLan = allowLan
        self.disableIpv6 = disableIpv6
        self.enableTwoHop = enableTwoHop
        self.enableBridges = enableBridges
        self.enableLewes = enableLewes
        self.netstack = netstack
        self.minGatewayVpnPerformance = minGatewayVpnPerformance
        self.residentialExit = residentialExit
        self.mixnetTuningConfig = mixnetTuningConfig
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
        self.enableLewes = config.enableLewesProtocol
        self.netstack = config.netstack
        self.minGatewayVpnPerformance = config.minGatewayVpnPerformance
        self.residentialExit = config.residentialExit
        self.mixnetTuningConfig = MixnetTuningConfig(from: config.mixnetTraffic)
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
        }
    }

    static func exitRouter(from exitPoint: ExitPoint) -> ExitRouter {
        switch exitPoint {
        case let .address(address):
            ExitRouter.address(address)
        case let .gateway(identity):
            ExitRouter.gateway(identity)
        case let .country(twoLetterIsoCountryCode):
            ExitRouter.country(twoLetterIsoCountryCode)
        case let .region(region):
            ExitRouter.region(countryCode: "", region: region)
        case .random:
            ExitRouter.random
        }
    }

    func entryPoint(from entryGateway: EntryGateway) -> EntryPoint {
        switch entryGateway {
        case let .country(code):
            EntryPoint.country(twoLetterIsoCountryCode: code)
        case let .lowLatencyCountry(code):
            EntryPoint.country(twoLetterIsoCountryCode: code)
        case let .gateway(node):
            EntryPoint.gateway(identity: node)
        case .random:
            EntryPoint.random
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
        case let .address(address):
            ExitPoint.address(address: address)
        case let .region(countryCode: _, region: region):
            ExitPoint.region(region: region)
        case .random:
            ExitPoint.random
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
        lhs.minGatewayVpnPerformance == rhs.minGatewayVpnPerformance &&
        lhs.residentialExit == rhs.residentialExit
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
