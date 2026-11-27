import NymVPNLib

extension GRPCManager {
    public func updateErrorReportingIfNeeded(with isEnabled: Bool) async throws {
        if isEnabled {
            try await client.enableSentry()
        } else {
            try await client.disableSentry()
        }
    }

    public func updateNetworkStatisticsIfNeeded(with isEnabled: Bool) async throws {
        try await client.networkStatsSetEnabled(enabled: isEnabled)
    }

    public func needFullDiskAccess() async throws -> Bool {
        try await client.needFullDiskPermissions()
    }

    public func runDiagnostic() async throws -> String? {
        try await client.runDiagnostic()
    }
}
