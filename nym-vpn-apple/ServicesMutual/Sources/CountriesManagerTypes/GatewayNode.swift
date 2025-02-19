public struct GatewayNode: Codable {
    public let id: String
    public let countryCode: String
    public let wgScore: GatewayNodeScore
    public let mixnetScore: GatewayNodeScore
    public let moniker: String?

    public init(
        id: String,
        countryCode: String,
        wgScore: GatewayNodeScore,
        mixnetScore: GatewayNodeScore,
        moniker: String? = nil
    ) {
        self.id = id
        self.countryCode = countryCode
        self.wgScore = wgScore
        self.mixnetScore = mixnetScore
        self.moniker = moniker
    }
}

extension GatewayNode: Equatable {
    public static func == (lhs: GatewayNode, rhs: GatewayNode) -> Bool {
        lhs.id == rhs.id
    }
}
