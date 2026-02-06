#if os(iOS)
import NymVPNLib
#elseif os(macOS)
import NymVPNRpc
#endif

public struct MixnetTuningConfig: Codable, Equatable {
    // 'background cover traffic' slider
    public var backgroundTraffic: BackgroundTraffic
    // Mixing delay
    public var averagePacketDelay = 25
    // 0.7, 1, 2 Mpbs
    public var continuousTraffic: ContinuousTraffic

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
        backgroundTraffic: BackgroundTraffic,
        continuousTraffic: ContinuousTraffic,
        dissablePoissonRate: Bool,
        averagePacketDelay: Int = 25
    ) {
        self.backgroundTraffic = backgroundTraffic
        self.averagePacketDelay = averagePacketDelay
        self.continuousTraffic = continuousTraffic
        self.disablePoissonRate = dissablePoissonRate
    }

    public init(from config: MixnetTrafficConfig) {
        self.backgroundTraffic = BackgroundTraffic(actualValue: config.poissonParameterForLoopCoverStream)
        self.averagePacketDelay = Int(config.averagePacketDelay ?? 25)
        self.continuousTraffic = ContinuousTraffic(actualValue: config.messageSendingAverageDelay)
        self.disablePoissonRate = config.disablePoissonRate
    }

    public func mixnetTrafficConfig() -> MixnetTrafficConfig {
        MixnetTrafficConfig(
            poissonParameterForLoopCoverStream: UInt32(backgroundTraffic.actualValue),
            averagePacketDelay: UInt32(averagePacketDelay),
            messageSendingAverageDelay: UInt32(continuousTraffic.actualValue),
            disablePoissonRate: disablePoissonRate,
            disableBackgroundCoverTraffic: false,
            minMixnodePerformance: nil,
            minGatewayMixnetPerformance: nil
        )
    }
}
