import NymVPNLib

public struct CompatibleVersions {
    public var macOS: String?
    public var core: String?
    public var policy: String
    public var policies: [String: String]
}

extension GRPCManager {
    public func fetchCompatibleVersions() async throws -> CompatibleVersions {
        guard let result = try await rpcClient?.getAppCompatibility() else {
            return CompatibleVersions(macOS: nil, core: nil, policy: "dismissible", policies: [:])
        }
        return CompatibleVersions(
            macOS: result.network?.macos,
            core: result.network?.core,
            policy: result.appUpdatePolicy,
            policies: result.appUpdatePolicies
        )
    }

    public func fetchFeatureFlags() async throws -> FeatureFlags? {
        try await rpcClient?.getFeatureFlags()
    }
}
