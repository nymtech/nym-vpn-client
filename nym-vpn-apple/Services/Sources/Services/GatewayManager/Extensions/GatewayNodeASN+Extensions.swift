#if os(iOS)
import CountriesManagerTypes
import NymVPNLib

extension GatewayNodeASN {
    init?(with asn: Asn?) {
        guard let asn else { return nil }
        self.init(asn: asn.asn, asnName: asn.name, type: GatewayNodeASNType(with: asn.kind))
    }
}
#endif
