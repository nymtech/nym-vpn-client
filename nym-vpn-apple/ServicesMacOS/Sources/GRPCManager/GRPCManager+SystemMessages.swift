import MessageModels
import FeatureFlagModels

extension GRPCManager {
    public func fetchSystemMessages() async throws -> [NymNetworkMessage] {
        guard let result = try await rpcClient?.getSystemMessages() else { return [] }
        return result.compactMap {
            NymNetworkMessage(name: $0.name, message: $0.message, properties: $0.properties)
        }
    }

    public func fetchCompatibleVersions() async throws -> (macOS: String?, core: String?) {
        guard let result = try await rpcClient?.getNetworkCompatibility() else { return (nil, nil)}
        return (macOS: result.macos, core: result.core)
    }

    public func fetchFeatureFlags() async throws -> [FeatureFlag] {
        let result = try await client.getFeatureFlags(Google_Protobuf_Empty())

        let topLevel = result.flags.map { FeatureFlag(name: $0.key, value: $0.value) }
        let grouped = result.groups.flatMap { groupName, group -> [FeatureFlag] in
            group.map.map { key, value in
                FeatureFlag(name: "\(groupName).\(key)", value: value)
            }
        }
        return topLevel + grouped
    }
}
