import CountriesManagerTypes

extension GRPCManager {
    public func gateways(for type: NodeType) async throws -> [GatewayNode] {
        guard let result = try await rpcClient?.listGateways(gwType: type.convertToGatewayType()) else { return [] }

        return result.compactMap {
            GatewayNode(with: $0)
        }
    }
}
