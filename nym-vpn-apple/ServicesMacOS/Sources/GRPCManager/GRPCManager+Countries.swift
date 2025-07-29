import CountriesManagerTypes

extension GRPCManager {
    public func countryCodes(for type: NodeType) async throws -> [String] {
        var request = NymVpnService_ListCountriesRequest()
        request.kind = type.convertToGatewayType()
        request.userAgent = userAgent

        return try await client.listCountries(request)
            .countries.map { $0.twoLetterIsoCountryCode }
    }
}
