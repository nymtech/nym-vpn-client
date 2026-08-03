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

    public func setEntryPoint(_ entry: EntryGateway) async throws {
        try await rpcClient?.setEntryPoint(entryPoint: entry.entryPoint)
    }

    public func setExitPoint(_ exit: ExitRouter) async throws {
        try await rpcClient?.setExitPoint(exitPoint: exit.exitPoint)
    }

    public func setEnableTwoHop(_ enabled: Bool) async throws {
        try await rpcClient?.setEnableTwoHop(enableTwoHop: enabled)
    }

    public func setEnableBridges(_ enabled: Bool) async throws {
        try await rpcClient?.setEnableBridges(enableBridges: enabled)
    }

    public func setMixnetTrafficConfig(_ config: MixnetTuningConfig) async throws {
        try await rpcClient?.setMixnetTrafficConfig(mixnetTrafficConfig: config.mixnetTrafficConfig())
    }

    public func setEnableAdBlocking(_ enabled: Bool) async throws {
        try await rpcClient?.setEnableAdBlocking(enableAdBlocking: enabled)
    }

    public func setStealthApiEnabled(_ enabled: Bool) async throws {
        try await rpcClient?.setFrontingMode(frontingMode: enabled ? .always : .onRetry)
    }

    public func setDisableIpv6(_ disabled: Bool) async throws {
        try await rpcClient?.setDisableIpv6(disableIpv6: disabled)
    }

    public func setAllowLan(_ allowed: Bool) async throws {
        try await rpcClient?.setAllowLan(allowLan: allowed)
    }

    public func setGatewayIndependence(_ isEnabled: Bool) async throws {
        try await rpcClient?.setEnableGatewayIndependence(enableGatewayIndependence: isEnabled)
    }

    public func setGatewayIndependenceNotifications(_ enabled: Bool) async throws {
        try await rpcClient?.setGatewayIndependenceNotifications(enableNotifications: enabled)
    }

    public func setSplitTunnelConfig(_ config: SplitTunnelConfig) async throws {
        let oldConfig = await self.config()?.splitTunnelConfig ?? SplitTunnelConfig()
        if oldConfig.isEnabled != config.isEnabled {
            try await rpcClient?.setEnableSplitTunnel(enable: config.isEnabled)
        }
        let diff = oldConfig.diff(comparedTo: config)
        for path in diff.removed {
            try await rpcClient?.removeSplitTunnelApp(app: SplitApp(path: path))
        }
        for path in diff.added {
            try await rpcClient?.addSplitTunnelApp(app: SplitApp(path: path))
        }
    }
}
