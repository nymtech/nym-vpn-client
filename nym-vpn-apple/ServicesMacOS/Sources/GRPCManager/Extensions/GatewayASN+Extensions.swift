import CountriesManagerTypes

extension GatewayASN {
    init(with asn: NymVpnService_Asn) {
        self.init(asn: asn.asn, asnName: asn.name, type: GatewayASNType(with: asn.kind))
    }
}
