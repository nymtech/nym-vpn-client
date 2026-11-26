import ConnectionTypes
import NymVPNLib
import TunnelStatus

extension GRPCManager {
    public func gateways(for type: NodeType) async throws -> [GatewayNode] {
        try await client.listGateways(gwType: type.convertToGatewayType()).compactMap { GatewayNode(with: $0) }
    }

    /// Gateways the daemon last connected through, most recent first.
    public func recentGateways(
        for tunnelType: ConnectionTunnelType
    ) async throws -> (entry: [GatewayNode], exit: [GatewayNode]) {
        let recents = try await client.getRecentGateways(tunnelType: tunnelType.rpcValue)
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
