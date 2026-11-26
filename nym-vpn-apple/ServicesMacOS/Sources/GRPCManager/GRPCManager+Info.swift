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

    public func isErrorReportingEnabled() async throws -> Bool {
        guard let rpcClient else { throw DaemonSettingError.notConnected }
        return try await rpcClient.isSentryEnabled()
    }

    public func isNetworkStatisticsEnabled() async throws -> Bool {
        guard let rpcClient else { throw DaemonSettingError.notConnected }
        return try await rpcClient.getConfig().networkStats.enabled
    }

    public func needFullDiskAccess() async throws -> Bool {
        guard let rpcClient else { return false }
        return try await rpcClient.needFullDiskPermissions()
    }

    public func runDiagnostic() async throws -> String? {
        try await rpcClient?.runDiagnostic()
    }
}
