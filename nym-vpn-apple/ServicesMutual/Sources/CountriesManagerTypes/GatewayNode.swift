public struct GatewayNode: Codable, Hashable {
    public let id: String
    public let countryCode: String
    public let city: String
    public let region: String
    public let asn: GatewayASN?
    public let performance: GatewayPerformance
    public let mixnetScore: GatewayNodeScore
    public let moniker: String?
    public let buildVersion: String?
    public let ipv4s: [String]
    public let ipv6s: [String]

    public init(
        id: String,
        countryCode: String,
        city: String,
        region: String,
        asn: GatewayASN?,
        performance: GatewayPerformance,
        mixnetScore: GatewayNodeScore,
        moniker: String?,
        buildVersion: String?,
        ipv4s: [String],
        ipv6s: [String]
    ) {
        self.id = id
        self.countryCode = countryCode
        self.city = city
        self.region = region
        self.asn = asn
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
