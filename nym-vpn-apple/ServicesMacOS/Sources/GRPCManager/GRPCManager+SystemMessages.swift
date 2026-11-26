import NymVPNLib

public struct CompatibleVersions {
    public var macOS: String?
    public var core: String?
    public var policy: String
    public var policies: [String: String]
}

extension GRPCManager {
    public func fetchCompatibleVersions() async throws -> CompatibleVersions {
        let result = try await client.getAppCompatibility()
        return CompatibleVersions(
            macOS: result.network?.macos,
            core: result.network?.core,
            policy: result.appUpdatePolicy,
            policies: result.appUpdatePolicies
        )
    }

    public func fetchFeatureFlags() async throws -> FeatureFlags? {
        try await client.getFeatureFlags()
    }
}
