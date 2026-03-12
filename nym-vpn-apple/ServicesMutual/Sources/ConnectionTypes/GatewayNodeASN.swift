public struct GatewayNodeASN: Codable, Hashable {
    public let asn: String
    public let asnName: String
    public let type: GatewayNodeASNType

    public init(asn: String, asnName: String, type: GatewayNodeASNType) {
        self.asn = asn
        self.asnName = asnName
        self.type = type
    }
}
