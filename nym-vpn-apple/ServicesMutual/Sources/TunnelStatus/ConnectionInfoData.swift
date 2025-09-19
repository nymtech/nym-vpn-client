public struct ConnectionInfoData: Codable, Equatable {
    public let entryGatewayId: String?
    public let exitGatewayId: String?

    public init(entryGatewayId: String?, exitGatewayId: String?) {
        self.entryGatewayId = entryGatewayId
        self.exitGatewayId = exitGatewayId
    }
}
