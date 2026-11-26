import ConnectionTypes
import NymVPNLib
import TunnelStatus

extension GRPCManager {
    public func gateways(for type: NodeType) async throws -> [GatewayNode] {
        guard let result = try await rpcClient?.listGateways(gwType: type.convertToGatewayType()) else {
            return []
        }
        return result.compactMap { GatewayNode(with: $0) }
    }

    /// Gateways the daemon last connected through, most recent first.
    public func recentGateways(
        for tunnelType: ConnectionTunnelType
    ) async throws -> (entry: [GatewayNode], exit: [GatewayNode]) {
        guard let recents = try await rpcClient?.getRecentGateways(tunnelType: tunnelType.rpcValue)
        else {
            return ([], [])
        }
        return (recents.entry.compactMap { GatewayNode(with: $0) }, recents.exit.compactMap { GatewayNode(with: $0) })
    }
}

private extension ConnectionTunnelType {
    var rpcValue: NymVPNLib.TunnelType {
        switch self {
        case .mixnet:
            .mixnet
        case .wireguard:
            .wireguard
        }
    }
}
