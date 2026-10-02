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

    // Background cover traffic toggle
    public var disableBackgroundCoverTraffic = false
    public var defaultDisableBackgroundCoverTraffic = false

    private enum CodingKeys: String, CodingKey {
        case backgroundTraffic
        case averagePacketDelay
        case continuousTraffic
        case disablePoissonRate
        case defaultDisablePoissonRate
        case disableBackgroundCoverTraffic
        case defaultDisableBackgroundCoverTraffic
    }

    /// Mixnet tuning config
    /// - Parameters:
    ///   - poissonParameterForLoopCoverStream: 'background cover traffic' slider
    ///   - averagePacketDelay: Mixing delay
    ///   - messageSendingAverageDelay: 0.7, 1, 2 Mbps
    ///   - dissablePoissonRate: Continous traffic toggle
    public init(
        backgroundTraffic: BackgroundCoverTrafficRate,
        continuousTraffic: ContinuousTrafficSendingRate,
        dissablePoissonRate: Bool,
        disableBackgroundCoverTraffic: Bool = false,
        averagePacketDelay: Int = 15
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

    // Custom decoder so a config persisted before `disableBackgroundCoverTraffic` existed
    // (pre-upgrade blobs lack the key) decodes cleanly instead of throwing `keyNotFound`,
    // which would fail the whole ConnectionConfig decode and reset ALL stored settings.
    // Mirrors the `decodeIfPresent ?? default` pattern ConnectionConfig uses for
    // stealthMode / geoExclusionConfig / gatewaySelectionAlgorithmConfig.
    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        self.backgroundTraffic = try container.decode(BackgroundCoverTrafficRate.self, forKey: .backgroundTraffic)
        self.averagePacketDelay = try container.decodeIfPresent(Int.self, forKey: .averagePacketDelay) ?? 15
        self.continuousTraffic = try container.decode(ContinuousTrafficSendingRate.self, forKey: .continuousTraffic)
        self.disablePoissonRate = try container.decode(Bool.self, forKey: .disablePoissonRate)
        self.defaultDisablePoissonRate = try container.decodeIfPresent(Bool.self, forKey: .defaultDisablePoissonRate) ?? false
        self.disableBackgroundCoverTraffic = try container.decodeIfPresent(Bool.self, forKey: .disableBackgroundCoverTraffic) ?? false
        self.defaultDisableBackgroundCoverTraffic = try container.decodeIfPresent(
            Bool.self,
            forKey: .defaultDisableBackgroundCoverTraffic
        ) ?? false
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
}
