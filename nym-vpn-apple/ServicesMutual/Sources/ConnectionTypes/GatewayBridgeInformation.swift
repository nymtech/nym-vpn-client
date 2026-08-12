import NymVPNLib

public struct GatewayBridgeInformation: Codable, Hashable {
    public var version: String
    public var transports: [GatewayBridgeParameters]

    public init(version: String, transports: [GatewayBridgeParameters]) {
        self.version = version
        self.transports = transports
    }
}

public extension GatewayBridgeInformation {
    init?(with info: PersistedClientConfig?) {
        guard let info else { return nil }
        self.init(version: info.version, transports: info.transports.map { GatewayBridgeParameters(with: $0) })
    }
}
