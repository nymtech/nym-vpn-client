import Shell

extension GRPCManager {
    public func version() async throws {
        try await Task.detached { [weak self] in
            do {
                guard let result = try await self?.rpcClient?.getInfo()
                else {
                    Task { @MainActor in
                        self?.daemonVersion = "noVersion"
                    }
                    return
                }
                Task { @MainActor in
                    self?.daemonVersion = result.version
                    self?.networkName = result.nymNetwork.networkName
                    self?.logger.info("🛜 \(result.nymNetwork.networkName)")
                }
            } catch {
                Task { @MainActor in
                    guard self?.daemonVersion != "noVersion" || self?.daemonVersion != "update" else { return }
                    self?.daemonVersion = "noVersion"
                }
                throw error
            }
        }.value
    }

    public func updateErrorReportingIfNeeded(with isEnabled: Bool) async throws {
        try await Task.detached { [weak self] in
            let isSentryEnabled = try await self?.rpcClient?.isSentryEnabled()
            guard isSentryEnabled != isEnabled else { return }
            if isEnabled {
                try await self?.rpcClient?.enableSentry()
            } else {
                try await self?.rpcClient?.disableSentry()
            }
        }.value
    }

    public func updateNetworkStatisticsIfNeeded(with isEnabled: Bool) async throws {
        try await Task.detached { [weak self] in
            let isStatisticsEnabled = try await self?.rpcClient?.isCollectNetworkStatsEnabled()
            guard isStatisticsEnabled != isEnabled else { return }
            if isEnabled {
                try await self?.rpcClient?.enableCollectNetworkStats()
            } else {
                try await self?.rpcClient?.disableCollectNetworkStats()
            }
        }.value
    }
}
