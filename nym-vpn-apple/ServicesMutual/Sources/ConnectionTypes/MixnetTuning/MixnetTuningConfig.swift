import NymVPNLib

public struct MixnetTuningConfig: Codable, Equatable {
    // 'background cover traffic' slider
    public var backgroundTraffic: BackgroundCoverTrafficRate
    // Mixing delay
    public var averagePacketDelay = 15
    // 0.7, 1, 2 Mpbs
    public var continuousTraffic: ContinuousTrafficSendingRate

    // Continous traffic toggle
    public var disablePoissonRate: Bool
    public var defaultDisablePoissonRate = false

//    public var disableBackgroundCoverTraffic: Bool

    /// Mixnet tuning config
    /// - Parameters:
    ///   - poissonParameterForLoopCoverStream: 'background cover traffic' slider
    ///   - averagePacketDelay: Mixing delay
    ///   - messageSendingAverageDelay: 0.7, 1, 2 Mpbs
    ///   - dissablePoissonRate: Continous traffic toggle
    public init(
        backgroundTraffic: BackgroundCoverTrafficRate,
        continuousTraffic: ContinuousTrafficSendingRate,
        dissablePoissonRate: Bool,
        averagePacketDelay: Int = 15
    ) {
        self.backgroundTraffic = backgroundTraffic
        self.averagePacketDelay = averagePacketDelay
        self.continuousTraffic = continuousTraffic
        self.disablePoissonRate = dissablePoissonRate
    }

    public init(from config: MixnetTrafficConfig) {
        self.backgroundTraffic = BackgroundCoverTrafficRate(fromValue: config.poissonParameterForLoopCoverStream)
        self.averagePacketDelay = Int(config.averagePacketDelay ?? 15)
        self.continuousTraffic = ContinuousTrafficSendingRate(fromValue: config.messageSendingAverageDelay)
        self.disablePoissonRate = config.disablePoissonRate
    }

    public func mixnetTrafficConfig() -> MixnetTrafficConfig {
        MixnetTrafficConfig(
            poissonParameterForLoopCoverStream: backgroundTraffic.value(),
            averagePacketDelay: UInt32(averagePacketDelay),
            messageSendingAverageDelay: continuousTraffic.value(),
            disablePoissonRate: disablePoissonRate,
            disableBackgroundCoverTraffic: false,
            minMixnodePerformance: nil,
            minGatewayMixnetPerformance: nil
        )
    }
}
