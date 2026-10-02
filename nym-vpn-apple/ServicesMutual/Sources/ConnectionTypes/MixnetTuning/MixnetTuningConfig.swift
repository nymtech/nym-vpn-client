import NymVPNLib

public struct MixnetTuningConfig: Codable, Equatable {
    // 'background cover traffic' slider
    public var backgroundTraffic: BackgroundCoverTrafficRate
    // Mixing delay
    public var averagePacketDelay = 15
    // 0.7, 1, 2 Mbps
    public var continuousTraffic: ContinuousTrafficSendingRate

    // Continuous traffic toggle
    public var disablePoissonRate: Bool
    public var defaultDisablePoissonRate = false

    // true = cover off. A stored config without this key stays false (cover on).
    public var disableBackgroundCoverTraffic: Bool

    /// Mixnet tuning config
    /// - Parameters:
    ///   - poissonParameterForLoopCoverStream: 'background cover traffic' slider
    ///   - averagePacketDelay: Mixing delay
    ///   - messageSendingAverageDelay: 0.7, 1, 2 Mbps
    ///   - dissablePoissonRate: Continuous traffic toggle
    public init(
        backgroundTraffic: BackgroundCoverTrafficRate,
        continuousTraffic: ContinuousTrafficSendingRate,
        dissablePoissonRate: Bool,
        averagePacketDelay: Int = 15,
        disableBackgroundCoverTraffic: Bool = false
    ) {
        self.backgroundTraffic = backgroundTraffic
        self.averagePacketDelay = averagePacketDelay
        self.continuousTraffic = continuousTraffic
        self.disablePoissonRate = dissablePoissonRate
        self.disableBackgroundCoverTraffic = disableBackgroundCoverTraffic
    }

    public init(from config: MixnetTrafficConfig) {
        self.backgroundTraffic = BackgroundCoverTrafficRate(fromValue: config.poissonParameterForLoopCoverStream)
        self.averagePacketDelay = Int(config.averagePacketDelay ?? 15)
        self.continuousTraffic = ContinuousTrafficSendingRate(fromValue: config.messageSendingAverageDelay)
        self.disablePoissonRate = config.disablePoissonRate
        self.disableBackgroundCoverTraffic = config.disableBackgroundCoverTraffic
    }

    public func mixnetTrafficConfig() -> MixnetTrafficConfig {
        MixnetTrafficConfig(
            poissonParameterForLoopCoverStream: backgroundTraffic.value(),
            averagePacketDelay: UInt32(averagePacketDelay),
            messageSendingAverageDelay: continuousTraffic.value(),
            disablePoissonRate: disablePoissonRate,
            disableBackgroundCoverTraffic: disableBackgroundCoverTraffic,
            minMixnodePerformance: nil,
            minGatewayMixnetPerformance: nil
        )
    }

    enum CodingKeys: String, CodingKey {
        case backgroundTraffic
        case averagePacketDelay
        case continuousTraffic
        case disablePoissonRate
        case defaultDisablePoissonRate
        case disableBackgroundCoverTraffic
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        backgroundTraffic = try container.decode(BackgroundCoverTrafficRate.self, forKey: .backgroundTraffic)
        averagePacketDelay = try container.decodeIfPresent(Int.self, forKey: .averagePacketDelay) ?? 15
        continuousTraffic = try container.decode(ContinuousTrafficSendingRate.self, forKey: .continuousTraffic)
        disablePoissonRate = try container.decode(Bool.self, forKey: .disablePoissonRate)
        defaultDisablePoissonRate = try container.decodeIfPresent(Bool.self, forKey: .defaultDisablePoissonRate) ?? false
        disableBackgroundCoverTraffic = try container.decodeIfPresent(Bool.self, forKey: .disableBackgroundCoverTraffic) ?? false
    }
}
