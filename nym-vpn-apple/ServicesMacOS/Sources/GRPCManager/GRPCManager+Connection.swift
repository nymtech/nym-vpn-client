import NymVPNRpc
import Constants
import ConnectionTypes

extension GRPCManager {
    public func config() async -> ConnectionConfig? {
        await Task.detached { [weak self] in
            guard let cfg = try? await self?.rpcClient?.getConfig() else { return nil }
            return ConnectionConfig(from: cfg)
        }.value
    }

    public func updateConfig(newConfig: ConnectionConfig) async throws {
        guard let oldConfig = await config() else { return }
        try await Task.detached { [weak self] in
            if oldConfig.entry != newConfig.entry {
                try await self?.rpcClient?.setEntryPoint(entryPoint: newConfig.entryPoint)
            }
            if oldConfig.exit != newConfig.exit {
                try await self?.rpcClient?.setExitPoint(exitPoint: newConfig.exitPoint)
            }
            if oldConfig.disableIpv6 != newConfig.disableIpv6 {
                try await self?.rpcClient?.setDisableIpv6(disableIpv6: newConfig.disableIpv6)
            }
            if oldConfig.enableTwoHop != newConfig.enableTwoHop {
                try await self?.rpcClient?.setEnableTwoHop(enableTwoHop: newConfig.enableTwoHop)
            }
            if oldConfig.enableBridges != newConfig.enableBridges {
                try await self?.rpcClient?.setEnableBridges(enableBridges: newConfig.enableBridges)
            }
            if oldConfig.allowLan != newConfig.allowLan {
                try await self?.rpcClient?.setAllowLan(allowLan: newConfig.allowLan)
            }
            if oldConfig.enableLewes != newConfig.enableLewes {
                try await self?.rpcClient?.setEnableLewesProtocol(enableLewesProtocol: newConfig.enableLewes)
            }
        }.value
    }

    public func connect() async throws {
        try await Task.detached { [weak self] in
            try await self?.rpcClient?.connectTunnel()
        }.value
    }

    public func disconnect() async throws {
        try await Task.detached { [weak self] in
            try await self?.rpcClient?.disconnectTunnel()
        }.value
    }
}
