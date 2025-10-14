import Foundation

public struct GatewayNodePerformance: Codable, Hashable {
    public let lastUpdated: Date?
    public let score: GatewayNodeScore
    public let mixnetScore: GatewayNodeScore
    public let load: GatewayNodeScore
    public let uptime: Double

    public init(
        lastUpdated: Date?,
        score: GatewayNodeScore,
        mixnetScore: GatewayNodeScore,
        load: GatewayNodeScore,
        uptime: Double
    ) {
        self.lastUpdated = lastUpdated
        self.score = score
        self.mixnetScore = mixnetScore
        self.load = load
        self.uptime = uptime
    }
}
