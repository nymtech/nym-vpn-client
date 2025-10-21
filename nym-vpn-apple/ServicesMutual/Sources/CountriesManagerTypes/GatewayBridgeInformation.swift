public struct GatewayBridgeInformation: Codable, Hashable {
    public var version: String
    public var transports: [GatewayBridgeParameters]

    public init(version: String, transports: [GatewayBridgeParameters]) {
        self.version = version
        self.transports = transports
    }
}

#if os(iOS)
import NymVPNLib

public extension GatewayBridgeInformation {
    init?(with info: BridgeInformation?) {
        guard let info else { return nil }
        self.init(version: info.version, transports: info.transports.map { GatewayBridgeParameters(with: $0) })
    }
}
#elseif os(macOS)
import NymVPNRpc
public extension GatewayBridgeInformation {
    init?(with info: BridgeInformation?) {
        guard let info else { return nil }
        self.init(version: info.version, transports: info.transports.map { GatewayBridgeParameters(with: $0) })
    }
}
#endif
