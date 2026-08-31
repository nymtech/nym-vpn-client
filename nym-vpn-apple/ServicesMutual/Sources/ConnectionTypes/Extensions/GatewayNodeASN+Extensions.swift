import NymVPNLib

extension GatewayNodeASN {
    public init?(with asn: Asn?) {
        guard let asn else { return nil }
        self.init(asn: asn.asn, asnName: asn.name, type: GatewayNodeASNType(with: asn.kind))
    }
}
