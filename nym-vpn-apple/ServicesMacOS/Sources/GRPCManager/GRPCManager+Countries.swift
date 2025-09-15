import CountriesManagerTypes

extension GRPCManager {
    public func countryCodes(for type: NodeType) async throws -> [String] {
        guard let countries = try await rpcClient?.listCountries(gwType: type.convertToGatewayType()) else { return [] }
        return countries
            .map { $0.isoCode }
    }
}
