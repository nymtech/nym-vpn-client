#if os(iOS)

#elseif os(macOS)
import NymVPNRpc
#endif

public final class ConnectionConfig {
    public var entry: EntryGateway
    public var exit: ExitRouter
    public var dns: String?
    public var allowLan: Bool
    public var disableIpv6: Bool
    public var enableTwoHop: Bool
    public var enableBridges: Bool
    public var netstack: Bool
    public var disablePoissonRate: Bool
    public var disableBackgroundCoverTraffic: Bool
    public var minMixnodePerformance: UInt8?
    public var minGatewayMixnetPerformance: UInt8?
    public var minGatewayVpnPerformance: UInt8?
    public var residentialExit: Bool

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
        dns: String? = nil,
        allowLan: Bool,
        disableIpv6: Bool,
        enableTwoHop: Bool,
        enableBridges: Bool,
        netstack: Bool,
        disablePoissonRate: Bool,
        disableBackgroundCoverTraffic: Bool,
        minMixnodePerformance: UInt8? = nil,
        minGatewayMixnetPerformance: UInt8? = nil,
        minGatewayVpnPerformance: UInt8? = nil,
        residentialExit: Bool
    ) {
        self.entry = entry
        self.exit = exit
        self.dns = dns
        self.allowLan = allowLan
        self.disableIpv6 = disableIpv6
        self.enableTwoHop = enableTwoHop
        self.enableBridges = enableBridges
        self.netstack = netstack
        self.disablePoissonRate = disablePoissonRate
        self.disableBackgroundCoverTraffic = disableBackgroundCoverTraffic
        self.minMixnodePerformance = minMixnodePerformance
        self.minGatewayMixnetPerformance = minGatewayMixnetPerformance
        self.minGatewayVpnPerformance = minGatewayVpnPerformance
        self.residentialExit = residentialExit
    }

#if os(iOS)

#elseif os(macOS)
    public init(from config: VpnServiceConfig) {
        self.entry = ConnectionConfig.entryGateway(from: config.entryPoint)
        self.exit = ConnectionConfig.exitRouter(from: config.exitPoint)
        self.dns = config.dns
        self.allowLan = config.allowLan
        self.disableIpv6 = config.disableIpv6
        self.enableTwoHop = config.enableTwoHop
        self.enableBridges = config.enableBridges
        self.netstack = config.netstack
        self.disablePoissonRate = config.disablePoissonRate
        self.disableBackgroundCoverTraffic = config.disableBackgroundCoverTraffic
        self.minMixnodePerformance = config.minMixnodePerformance
        self.minGatewayMixnetPerformance = config.minGatewayMixnetPerformance
        self.minGatewayVpnPerformance = config.minGatewayVpnPerformance
        self.residentialExit = config.residentialExit
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
        case let .region(region):
            return .region(region)
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
            ExitRouter.region(region)
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
        case let .region(region):
            EntryPoint.region(region: region)
        case .city:
            EntryPoint.random
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
        case let .region(region):
            ExitPoint.region(region: region)
        case .random:
            ExitPoint.random
        }
    }
}
#endif
