import NymVPNLib

public struct CompatibleVersions: Equatable {
    public var macOS: String?
    public var core: String?
    public var policy: String
}

extension GRPCManager {
    public func fetchCompatibleVersions() async throws -> CompatibleVersions {
        try await Task.detached { [weak self] in
            guard let result = try await self?.rpcClient?.getNetworkCompatibility() else {
                return CompatibleVersions(macOS: nil, core: nil, policy: "dismissible")
            }
            return CompatibleVersions(
                macOS: result.macos,
                core: result.core,
                policy: result.appUpdatePolicy
            )
        }.value
    }

    public func fetchFeatureFlags() async throws -> FeatureFlags? {
        try await Task.detached { [weak self] in
            try await self?.rpcClient?.getFeatureFlags()
        }.value
    }
}
