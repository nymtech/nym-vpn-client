import Foundation
import NymVPNLib

extension GatewayNodePerformance {
    public init(with performance: Performance?) {
        self.init(
            lastUpdated: ISO8601DateFormatter().date(from: performance?.lastUpdatedUtc ?? ""),
            score: GatewayNodeScore(with: performance?.score),
            mixnetScore: GatewayNodeScore(with: performance?.mixnetScore),
            load: GatewayNodeScore(with: performance?.load),
            uptime: Double(performance?.uptimePercentageLast24Hours ?? 0)
        )
    }
}
