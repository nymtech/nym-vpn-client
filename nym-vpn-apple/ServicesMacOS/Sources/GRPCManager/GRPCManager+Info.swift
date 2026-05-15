extension GRPCManager {
    public func version() async throws {
        do {
            guard let result = try await rpcClient?.getInfo()
            else {
                Task { @MainActor in
                    daemonVersion = "noVersion"
                }
                return
            }
            Task { @MainActor in
                daemonVersion = result.version
                networkName = result.nymNetwork.networkName
                logger.info("🛜 \(result.nymNetwork.networkName)")
            }
        } catch {
            Task { @MainActor in
                guard daemonVersion != "noVersion" || daemonVersion != "update" else { return }
                daemonVersion = "noVersion"
            }
            throw error
        }
    }

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
