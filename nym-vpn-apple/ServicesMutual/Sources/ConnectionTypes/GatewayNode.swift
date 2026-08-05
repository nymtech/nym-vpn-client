public struct GatewayNode: Codable, Hashable {
    public let id: String
    public let location: GatewayNodeLocation?
    public let performance: GatewayNodePerformance?
    public let mixnetScore: GatewayNodeScore
    public let name: String?
    public let description: String?
    public let buildVersion: String?
    public let ipv4s: [String]
    public let ipv6s: [String]
    public let bridges: GatewayBridgeInformation?
    public let operatorFamilyName: String?

    public var isQuicAvailable: Bool {
        guard let bridges else { return false }
        return bridges.transports.contains {
            switch $0 {
            case .quicPlain:
                true
            case .tlsPlain:
                false
            }
        }
    }

    public var isResidentialAvailable: Bool {
        guard let location = location else { return false }
        return location.asn?.type == .residential
    }

    public init(
        id: String,
        location: GatewayNodeLocation?,
        performance: GatewayNodePerformance?,
        mixnetScore: GatewayNodeScore,
        name: String?,
        description: String?,
        buildVersion: String?,
        ipv4s: [String],
        ipv6s: [String],
        bridges: GatewayBridgeInformation?,
        operatorFamilyName: String? = nil
    ) {
        self.id = id
        self.location = location
        self.performance = performance
        self.mixnetScore = mixnetScore
        self.name = name
        self.description = description
        self.buildVersion = buildVersion
        self.ipv4s = ipv4s
        self.ipv6s = ipv6s
        self.bridges = bridges
        self.operatorFamilyName = operatorFamilyName
    }
}

extension GatewayNode: Equatable {
    public static func == (lhs: GatewayNode, rhs: GatewayNode) -> Bool {
        lhs.id == rhs.id
    }
}
