import MessageModels
import FeatureFlagModels

extension GRPCManager {
    public func fetchSystemMessages() async throws -> [NymNetworkMessage] {
        try await Task.detached { [weak self] in
            guard let result = try await self?.rpcClient?.getSystemMessages() else { return [] }
            return result.compactMap {
                NymNetworkMessage(name: $0.name, message: $0.message, properties: $0.properties)
            }
        }.value
    }

    public func fetchCompatibleVersions() async throws -> (macOS: String?, core: String?) {
        try await Task.detached { [weak self] in
            guard let result = try await self?.rpcClient?.getNetworkCompatibility() else { return (nil, nil)}
            return (macOS: result.macos, core: result.core)
        }.value
    }

    public func fetchFeatureFlags() async throws -> [FeatureFlag] {
        try await Task.detached { [weak self] in
            guard let result = try await self?.rpcClient?.getFeatureFlags() else { return [] }

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
        }.value
    }
}
