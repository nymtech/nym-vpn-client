import NymVPNLib

extension GRPCManager {
    public func setErrorReporting(_ isEnabled: Bool) async throws {
        guard let rpcClient else { throw AccountDaemonRefreshError.clientUnavailable }
        if isEnabled {
            try await rpcClient.enableSentry()
        } else {
            try await rpcClient.disableSentry()
        }
    }

    public func setNetworkStatistics(_ isEnabled: Bool) async throws {
        guard let rpcClient else { throw AccountDaemonRefreshError.clientUnavailable }
        try await rpcClient.networkStatsSetEnabled(enabled: isEnabled)
    }

    public func isErrorReportingEnabled() async throws -> Bool {
        guard let rpcClient else { throw AccountDaemonRefreshError.clientUnavailable }
        return try await rpcClient.isSentryEnabled()
    }

    public func isNetworkStatisticsEnabled() async throws -> Bool {
        guard let rpcClient else { throw AccountDaemonRefreshError.clientUnavailable }
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
