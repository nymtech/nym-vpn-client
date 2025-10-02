public struct GatewayNode: Codable, Hashable {
    public let id: String
    public let location: GatewayNodeLocation?
    public let performance: GatewayNodePerformance?
    public let mixnetScore: GatewayNodeScore
    public let moniker: String?
    public let buildVersion: String?
    public let ipv4s: [String]
    public let ipv6s: [String]

    public init(
        id: String,
        location: GatewayNodeLocation?,
        performance: GatewayNodePerformance?,
        mixnetScore: GatewayNodeScore,
        moniker: String?,
        buildVersion: String?,
        ipv4s: [String],
        ipv6s: [String]
    ) {
        self.id = id
        self.location = location
        self.performance = performance
        self.mixnetScore = mixnetScore
        self.moniker = moniker
        self.buildVersion = buildVersion
        self.ipv4s = ipv4s
        self.ipv6s = ipv6s
    }
}

extension GatewayNode: Equatable {
    public static func == (lhs: GatewayNode, rhs: GatewayNode) -> Bool {
        lhs.id == rhs.id
    }
}
