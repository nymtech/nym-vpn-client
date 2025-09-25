import Foundation
import CountriesManagerTypes

extension GatewayPerformance {
    init(with performance: NymVpnService_Performance) {
        self.init(
            lastUpdated: ISO8601DateFormatter().date(from: performance.lastUpdatedUtc),
            score: GatewayNodeScore(with: performance.score),
            load: GatewayNodeScore(with: performance.load),
            uptime: Double(performance.uptimePercentageLast24Hours)
        )
    }
}
