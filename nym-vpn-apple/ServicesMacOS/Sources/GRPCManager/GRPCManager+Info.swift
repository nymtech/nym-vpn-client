import NymVPNLib

public enum DaemonSettingError: Error {
    case notConnected
}

extension GRPCManager {
    /// Throws `DaemonSettingError.notConnected` rather than returning silently, so a caller
    /// never mistakes "not sent" for "sent".
    public func setErrorReporting(_ isEnabled: Bool) async throws {
        guard let rpcClient else { throw DaemonSettingError.notConnected }
        if isEnabled {
            try await rpcClient.enableSentry()
        } else {
            try await rpcClient.disableSentry()
        }
    }

    public func setNetworkStatistics(_ isEnabled: Bool) async throws {
        guard let rpcClient else { throw DaemonSettingError.notConnected }
        try await rpcClient.networkStatsSetEnabled(enabled: isEnabled)
    }

    /// The daemon's error-reporting setting, or `nil` when not connected.
    public func isErrorReportingEnabled() async throws -> Bool? {
        try await rpcClient?.isSentryEnabled()
    }

    /// The daemon's network-statistics setting, or `nil` when not connected or unreadable.
    public func isNetworkStatisticsEnabled() async -> Bool? {
        guard let config = try? await rpcClient?.getConfig() else { return nil }
        return config.networkStats.enabled
    }

    public func needFullDiskAccess() async throws -> Bool {
        guard let rpcClient else { return false }
        return try await rpcClient.needFullDiskPermissions()
    }

    public func runDiagnostic() async throws -> String? {
        try await rpcClient?.runDiagnostic()
    }
}
