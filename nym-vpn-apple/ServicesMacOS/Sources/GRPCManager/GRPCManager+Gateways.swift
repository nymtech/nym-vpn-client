import CountriesManagerTypes

extension GRPCManager {
    public func gateways(for type: NodeType) async throws -> [GatewayNode] {
        var request = NymVpnService_ListGatewaysRequest()
        request.kind = type.convertToGatewayType()
        request.userAgent = userAgent

        let result = try await client.listGateways(request)
        return result.gateways.compactMap {
            GatewayNode(with: $0)
        }
    }
}
