public struct GatewayNodeLocation: Codable, Hashable {
    public let twoLetterIsoCountryCode: String
    public let latitude: Double
    public let longitude: Double
    public let city: String
    public let region: String
    public let asn: GatewayNodeASN?

    public init(
        twoLetterIsoCountryCode: String,
        latitude: Double,
        longitude: Double,
        city: String,
        region: String,
        asn: GatewayNodeASN?
    ) {
        self.twoLetterIsoCountryCode = twoLetterIsoCountryCode
        self.latitude = latitude
        self.longitude = longitude
        self.city = city
        self.region = region
        self.asn = asn
    }
}
