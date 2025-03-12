import CountriesManagerTypes

extension GRPCManager {
    public func countryCodes(for type: NodeType) async throws -> [String] {
        logger.log(level: .info, "Fetching countries: \(type.rawValue)")
        return try await withCheckedThrowingContinuation { continuation in

            var request = Nym_Vpn_ListCountriesRequest()
            request.kind = type.convertToGatewayType()
            request.userAgent = userAgent

            let call = client.listCountries(request, callOptions: nil)
            call.response.whenComplete { result in
                switch result {
                case let .success(countries):
                    continuation.resume(returning: countries.countries.map { $0.twoLetterIsoCountryCode })
                case let .failure(error):
                    continuation.resume(throwing: error)
                }
            }

            call.status.whenComplete { [weak self] result in
                switch result {
                case .success:
                    break
                case let .failure(error):
                    self?.logger.log(level: .error, "\(error.localizedDescription)")
                }
            }
        }
    }
}
