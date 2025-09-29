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
        guard let result = try await rpcClient?.getFeatureFlags() else { return [] }

        var list: [FeatureFlag] = []

        result.flags.forEach { name, flag in
            switch flag {
            case let .value(value):
                list.append(FeatureFlag(name: name, value: value))
            case let .group(dict):
                dict.forEach { key, value in
                    list.append(FeatureFlag(name: "\(name).\(key)", value: value))
                }
            }
        }
        list.sort { $0.name < $1.name }
        return list
    }
}
