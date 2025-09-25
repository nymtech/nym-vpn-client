public struct GatewayASN: Codable, Hashable {
    public let asn: String
    public let asnName: String
    public let type: GatewayASNType

    public init(asn: String, asnName: String, type: GatewayASNType) {
        self.asn = asn
        self.asnName = asnName
        self.type = type
    }
}
