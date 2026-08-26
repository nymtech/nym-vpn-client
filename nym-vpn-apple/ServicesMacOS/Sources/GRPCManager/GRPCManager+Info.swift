import NymVPNRpc

extension GRPCManager {
    public func updateErrorReportingIfNeeded(with isEnabled: Bool) async throws {
        if isEnabled {
            try await rpcClient?.enableSentry()
        } else {
            try await rpcClient?.disableSentry()
        }
    }

    public func updateNetworkStatisticsIfNeeded(with isEnabled: Bool) async throws {
        try await rpcClient?.networkStatsSetEnabled(enabled: isEnabled)
    }

    public func needFullDiskAccess() async throws -> Bool {
        guard let rpcClient else { return false }
        return try await rpcClient.needFullDiskPermissions()
    }

    public func runDiagnostic() async throws -> String? {
        try await rpcClient?.runDiagnostic()
    }
}
