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

    public func updateConfig() async throws {
//        try await rpcClient.set
    }

    public func config() async throws {
        let config = try await rpcClient?.getConfig()
    }

    public func connect() async throws {
        try await rpcClient?.connectTunnel()
    }

    public func disconnect() async throws {
        try await rpcClient?.disconnectTunnel()
    }
}
