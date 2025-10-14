import Foundation
import CountriesManagerTypes
import NymVPNRpc

extension GatewayNodePerformance {
    init?(with performance: Performance?) {
        guard let performance else { return nil }
        self.init(
            lastUpdated: ISO8601DateFormatter().date(from: performance.lastUpdatedUtc),
            score: GatewayNodeScore.convert(from: performance.score) ?? .noScore,
            mixnetScore: GatewayNodeScore.convert(from: performance.mixnetScore) ?? .noScore,
            load: GatewayNodeScore.convert(from: performance.load) ?? .noScore,
            uptime: Double(performance.uptimePercentageLast24Hours)
        )
    }
}
