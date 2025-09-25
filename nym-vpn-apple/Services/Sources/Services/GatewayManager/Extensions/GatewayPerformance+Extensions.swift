#if os(iOS)
import Foundation
import CountriesManagerTypes
import NymVPNLib

extension GatewayPerformance {
    init(with performance: Performance?) {
        self.init(
            lastUpdated: ISO8601DateFormatter().date(from: performance?.lastUpdatedUtc ?? ""),
            score: GatewayNodeScore(with: performance?.score),
            load: GatewayNodeScore(with: performance?.load),
            uptime: Double(performance?.uptimePercentageLast24Hours ?? 0)
        )
    }
}
#endif
