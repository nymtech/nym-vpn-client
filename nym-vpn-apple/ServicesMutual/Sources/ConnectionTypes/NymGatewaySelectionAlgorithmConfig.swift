import NymVPNLib

public struct NymGatewaySelectionAlgorithmConfig: Codable, Equatable, Sendable {
    public var enableGeoLocation: Bool

    public init(
        enableGeoLocation: Bool = true
    ) {
        self.enableGeoLocation = enableGeoLocation
    }
}

extension NymGatewaySelectionAlgorithmConfig {
    public init(from sdk: GatewaySelectionAlgorithmConfig) {
        self.enableGeoLocation = sdk.enableGeoLocation
    }

    public var sdkValue: GatewaySelectionAlgorithmConfig {
        GatewaySelectionAlgorithmConfig(
            enableGeoLocation: enableGeoLocation
        )
    }
}
