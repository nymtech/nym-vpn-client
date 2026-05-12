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

        if oldConfig.enableBridges != newConfig.enableBridges {
            try await rpcClient?.setEnableBridges(enableBridges: newConfig.enableBridges)
        }

        if oldConfig.allowLan != newConfig.allowLan {
            try await rpcClient?.setAllowLan(allowLan: newConfig.allowLan)
        }

        if oldConfig.enableAdBlocking != newConfig.enableAdBlocking {
            try await rpcClient?.setEnableAdBlocking(enableAdBlocking: newConfig.enableAdBlocking)
        }

        if oldConfig.mixnetTuningConfig != newConfig.mixnetTuningConfig {
            try await rpcClient?.setMixnetTrafficConfig(
                mixnetTrafficConfig: newConfig.mixnetTuningConfig.mixnetTrafficConfig()
            )
        }

        if oldConfig.splitTunnelConfig != newConfig.splitTunnelConfig {
            try await updateSplitTunnelConfig(
                oldConfig: oldConfig.splitTunnelConfig,
                newConfig: newConfig.splitTunnelConfig
            )
        }
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

    public func setEnableAdBlocking(_ enabled: Bool) async throws {
        try await rpcClient?.setEnableAdBlocking(enableAdBlocking: enabled)
    }

    public func setDisableIpv6(_ disabled: Bool) async throws {
        try await rpcClient?.setDisableIpv6(disableIpv6: disabled)
    }

    public func setAllowLan(_ allowed: Bool) async throws {
        try await rpcClient?.setAllowLan(allowLan: allowed)
    }
}

private extension GRPCManager {
    private func updateSplitTunnelConfig(
        oldConfig: SplitTunnelConfig,
        newConfig: SplitTunnelConfig
    ) async throws {
        if oldConfig.isEnabled != newConfig.isEnabled {
            try await rpcClient?.setEnableSplitTunnel(enable: newConfig.isEnabled)
        }

        let diff = oldConfig.diff(comparedTo: newConfig)

        for path in diff.removed {
            try await rpcClient?.removeSplitTunnelApp(app: SplitApp(path: path))
        }

        for path in diff.added {
            try await rpcClient?.addSplitTunnelApp(app: SplitApp(path: path))
        }
    }
}
