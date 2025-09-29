import CountriesManagerTypes
import NymVPNRpc

extension GatewayASN {
    init(with asn: GatewayAsn) {
        self.init(asn: asn.asn, asnName: asn.name, type: GatewayASNType(with: asn.kind))
    }
}
