#if os(iOS)
import NymVPNLib
#elseif os(macOS)
import NymVPNRpc
#endif

public struct NymGatewaySelectionAlgorithmConfig: Codable, Equatable, Sendable {
    public var enableGeoLocation: Bool
    public var algorithm: NymGatewaySelectionAlgorithm

    public init(
        enableGeoLocation: Bool = true,
        algorithm: NymGatewaySelectionAlgorithm = .auto
    ) {
        self.enableGeoLocation = enableGeoLocation
        self.algorithm = algorithm
    }
}

extension NymGatewaySelectionAlgorithmConfig {
    public init(from sdk: GatewaySelectionAlgorithmConfig) {
        self.enableGeoLocation = sdk.enableGeoLocation
        self.algorithm = NymGatewaySelectionAlgorithm(from: sdk.gatewaySelectionAlgorithm)
    }

    public var sdkValue: GatewaySelectionAlgorithmConfig {
        GatewaySelectionAlgorithmConfig(
            enableGeoLocation: enableGeoLocation,
            gatewaySelectionAlgorithm: algorithm.sdkValue
        )
    }
}
