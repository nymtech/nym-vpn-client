import NymVPNRpc
import Constants
import ConnectionTypes

extension GRPCManager {
    public func config() async -> ConnectionConfig? {
        guard let config = try? await rpcClient?.getConfig() else { return nil }
        return ConnectionConfig(from: config)
    }

    public func updateConfig(newConfig: ConnectionConfig) async throws {
        guard let oldConfig = await config() else { return }
        if oldConfig.entry != newConfig.entry {
            try await rpcClient?.setEntryPoint(entryPoint: newConfig.entryPoint)
        }
        if oldConfig.exit != newConfig.exit {
            try await rpcClient?.setExitPoint(exitPoint: newConfig.exitPoint)
        }
        if oldConfig.disableIpv6 != newConfig.disableIpv6 {
            try await rpcClient?.setDisableIpv6(disableIpv6: newConfig.disableIpv6)
        }
        if oldConfig.enableTwoHop != newConfig.enableTwoHop {
            try await rpcClient?.setEnableTwoHop(enableTwoHop: newConfig.enableTwoHop)
        }
    }

    public func connect() async throws {
        try await rpcClient?.connectTunnel()
    }

    public func disconnect() async throws {
        try await rpcClient?.disconnectTunnel()
    }
}
