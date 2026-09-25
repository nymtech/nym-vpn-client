import NymVPNLib

extension GRPCManager {
    public func fetchCompatibleVersions() async throws -> (macOS: String?, core: String?, policy: String) {
        try await Task.detached { [weak self] in
            guard let result = try await self?.rpcClient?.getNetworkCompatibility() else {
                return (nil, nil, "dismissible")
            }
            return (macOS: result.macos, core: result.core, policy: result.appUpdatePolicy)
        }.value
    }

    public func fetchFeatureFlags() async throws -> FeatureFlags? {
        try await Task.detached { [weak self] in
            try await self?.rpcClient?.getFeatureFlags()
        }.value
    }
}
