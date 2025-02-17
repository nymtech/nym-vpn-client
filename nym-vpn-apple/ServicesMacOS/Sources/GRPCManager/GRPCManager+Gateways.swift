import CountriesManagerTypes

extension GRPCManager {
    public func gateways(for type: NodeType) async throws -> [String] {
        logger.log(level: .info, "Fetching countries: \(type.rawValue)")
        return try await withCheckedThrowingContinuation { continuation in

            var request = Nym_Vpn_ListGatewaysRequest()
            request.kind = type.convertToGatewayType()
            request.userAgent = userAgent

            let call = client.listGateways(request, callOptions: nil)
            call.response.whenComplete { result in
                switch result {
                case let .success(gateways):
                    print(gateways)
                    continuation.resume(returning: [])
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
