import NymVPNLib
import Constants
import ConnectionTypes

extension GRPCManager {
    public func config() async -> ConnectionConfig? {
        guard let cfg = try? await client.getConfig() else { return nil }
        return ConnectionConfig(from: cfg)
    }

    public func connect() async throws {
        try await client.connectTunnel()
    }

    public func disconnect() async throws {
        try await client.disconnectTunnel()
    }

    public func setEntryPoint(_ entry: EntryGateway) async throws {
        try await client.setEntryPoint(entryPoint: entry.entryPoint)
    }

    public func setExitPoint(_ exit: ExitRouter) async throws {
        try await client.setExitPoint(exitPoint: exit.exitPoint)
    }

    public func setEnableTwoHop(_ enabled: Bool) async throws {
        try await client.setEnableTwoHop(enableTwoHop: enabled)
    }

    public func setBridgeMode(_ mode: BridgeModeSetting) async throws {
        try await client.setBridgeMode(bridgeMode: mode.bridgeMode)
    }

    public func setMixnetTrafficConfig(_ config: MixnetTuningConfig) async throws {
        try await client.setMixnetTrafficConfig(mixnetTrafficConfig: config.mixnetTrafficConfig())
    }

    public func setEnableAdBlocking(_ enabled: Bool) async throws {
        try await client.setEnableAdBlocking(enableAdBlocking: enabled)
    }

    public func setStealthApiEnabled(_ enabled: Bool) async throws {
        try await client.setFrontingMode(frontingMode: enabled ? .always : .onRetry)
    }

    public func setTransportPreference(_ preference: TransportPreferenceSetting) async throws {
        try await client.setTransportPreference(transportPreference: preference.bridgeTransport)
    }

    public func setProfile(_ profile: ConnectionProfile) async throws {
        try await client.setProfile(profile: profile.toCoreProfile())
    }

    public func setDisableIpv6(_ disabled: Bool) async throws {
        try await client.setDisableIpv6(disableIpv6: disabled)
    }

    public func setAllowLan(_ allowed: Bool) async throws {
        try await client.setAllowLan(allowLan: allowed)
    }

    public func setGatewayIndependence(_ isEnabled: Bool) async throws {
        try await client.setEnableGatewayIndependence(enableGatewayIndependence: isEnabled)
    }

    public func setGatewayIndependenceNotifications(_ enabled: Bool) async throws {
        try await client.setGatewayIndependenceNotifications(enableNotifications: enabled)
    }

    public func setSplitTunnelConfig(_ config: SplitTunnelConfig) async throws {
        let oldConfig = await self.config()?.splitTunnelConfig ?? SplitTunnelConfig()
        if oldConfig.isEnabled != config.isEnabled {
            try await client.setEnableSplitTunnel(enable: config.isEnabled)
        }
        let diff = oldConfig.diff(comparedTo: config)
        for path in diff.removed {
            try await client.removeSplitTunnelApp(app: SplitApp(path: path))
        }
        for path in diff.added {
            try await client.addSplitTunnelApp(app: SplitApp(path: path))
        }
    }
}
