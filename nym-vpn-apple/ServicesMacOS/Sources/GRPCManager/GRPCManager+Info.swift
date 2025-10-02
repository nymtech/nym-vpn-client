import Shell

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
        let isSentryEnabled = try await rpcClient?.isSentryEnabled()
        guard isSentryEnabled != isEnabled else { return }
        if isEnabled {
            try await rpcClient?.enableSentry()
        } else {
            try await rpcClient?.disableSentry()
        }
    }

    public func updateNetworkStatisticsIfNeeded(with isEnabled: Bool) async throws {
        let isStatisticsEnabled = try await rpcClient?.isCollectNetworkStatsEnabled()
        guard isStatisticsEnabled != isEnabled else { return }
        if isEnabled {
            try await rpcClient?.enableCollectNetworkStats()
        } else {
            try await rpcClient?.disableCollectNetworkStats()
        }
    }
}
