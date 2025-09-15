import NymVPNRpc
import Constants
import ConnectionTypes

extension GRPCManager {
    public func connect(
        entryGateway: EntryGateway,
        exitRouter: ExitRouter,
        isTwoHopEnabled: Bool,
        disableIPv6: Bool
    ) async throws {
//        var request = NymVpnService_ConnectRequest()
//        request.userAgent = userAgent
//
//        request.entry = entryNode(from: entryGateway)
//        request.exit = exitNode(from: exitRouter)
//
//        request.enableTwoHop = isTwoHopEnabled
//        request.disableBackgroundCoverTraffic = false
//        request.disableIpv6 = disableIPv6
//
//        _ = try await client.connectTunnel(request)
    }

    public func disconnect() async throws {
        try await rpcClient?.disconnectTunnel()
    }
}

private extension GRPCManager {
    // TODO: add lowLatencyCountry support
    func entryNode(from entryGateway: EntryGateway) -> EntryPoint {
        switch entryGateway {
        case let .country(country):
            EntryPoint.location(location: country.code)
        case let .lowLatencyCountry(country):
            EntryPoint.location(location: country.code)
        case let .gateway(node):
            EntryPoint.gateway(identity: node.id)
        case .random:
            EntryPoint.random
        }
    }

    func exitNode(from exitRouter: ExitRouter) -> ExitPoint {
        switch exitRouter {
        case let .country(country):
            ExitPoint.location(location: country.code)
        case let .gateway(node):
            ExitPoint.gateway(identity: node.id)
        }
    }
}
