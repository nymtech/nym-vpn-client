import NymVPNLib

extension GRPCManager {
    public func fetchCompatibleVersions() async throws -> (macOS: String?, core: String?) {
        try await Task.detached { [weak self] in
            guard let result = try await self?.rpcClient?.getNetworkCompatibility() else { return (nil, nil)}
            return (macOS: result.macos, core: result.core)
        }.value
    }

    public func fetchFeatureFlags() async throws -> FeatureFlags? {
        try await Task.detached { [weak self] in
            return try await self?.rpcClient?.getFeatureFlags()
        }.value
    }
}
