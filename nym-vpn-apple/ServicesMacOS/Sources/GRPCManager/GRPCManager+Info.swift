import NymVPNLib

extension GRPCManager {
    public func setErrorReporting(_ isEnabled: Bool) async throws {
        if isEnabled {
            try await client.enableSentry()
        } else {
            try await client.disableSentry()
        }
    }

    public func setNetworkStatistics(_ isEnabled: Bool) async throws {
        try await client.networkStatsSetEnabled(enabled: isEnabled)
    }

    public func isErrorReportingEnabled() async throws -> Bool {
        try await client.isSentryEnabled()
    }

    public func isNetworkStatisticsEnabled() async throws -> Bool {
        try await client.getConfig().networkStats.enabled
    }

    public func needFullDiskAccess() async throws -> Bool {
        try await client.needFullDiskPermissions()
    }

    public func runDiagnostic() async throws -> String? {
        try await client.runDiagnostic()
    }
}
