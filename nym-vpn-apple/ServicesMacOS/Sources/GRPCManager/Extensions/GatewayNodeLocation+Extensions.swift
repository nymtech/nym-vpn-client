import NymVPNRpc
import CountriesManagerTypes

extension GatewayNodeLocation {
    init?(with location: GatewayLocation?) {
        guard let location else { return nil }
        self.init(
            twoLetterIsoCountryCode: location.twoLetterIsoCountryCode,
            latitude: location.latitude,
            longitude: location.longitude,
            city: location.city,
            region: location.region,
            asn: GatewayNodeASN(with: location.asn)
        )
    }
}
