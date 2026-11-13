public struct ConnectionInfoData: Codable, Equatable {
    public let entryGatewayId: String?
    public let exitGatewayId: String?
    public let tunnelType: ConnectionTunnelType?
    public let bridgeTransport: String?

    public init(
        entryGatewayId: String?,
        exitGatewayId: String?,
        tunnelType: ConnectionTunnelType? = nil,
        bridgeTransport: String? = nil
    ) {
        self.entryGatewayId = entryGatewayId
        self.exitGatewayId = exitGatewayId
        self.tunnelType = tunnelType
        self.bridgeTransport = bridgeTransport
    }
}

public enum ConnectionTunnelType: String, Codable, Sendable {
    case mixnet
    case wireguard
}
