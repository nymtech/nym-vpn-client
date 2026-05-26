public struct ConnectionInfoData: Codable, Equatable {
    public let entryGatewayId: String?
    public let exitGatewayId: String?
    public let tunnelType: ConnectionTunnelType?

    public init(
        entryGatewayId: String?,
        exitGatewayId: String?,
        tunnelType: ConnectionTunnelType? = nil
    ) {
        self.entryGatewayId = entryGatewayId
        self.exitGatewayId = exitGatewayId
        self.tunnelType = tunnelType
    }
}

public enum ConnectionTunnelType: String, Codable, Sendable {
    case mixnet
    case wireguard
}
