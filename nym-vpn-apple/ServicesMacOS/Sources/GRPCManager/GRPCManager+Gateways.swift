import ConnectionTypes

extension GRPCManager {
    public func gateways(for type: NodeType) async throws -> [GatewayNode] {
        try await Task.detached { [weak self] in
            guard let result = try await self?.rpcClient?.listGateways(gwType: type.convertToGatewayType())
            else {
                return []
            }

            return result.compactMap {
                GatewayNode(with: $0)
            }
        }.value
    }
}
