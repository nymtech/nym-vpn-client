#if os(iOS)
import CountriesManagerTypes
import NymVPNLib

extension GatewayASN {
    init?(with asn: Asn?) {
        guard let asn else { return nil }
        self.init(asn: asn.asn, asnName: asn.name, type: GatewayASNType(with: asn.kind))
    }
}
#endif
