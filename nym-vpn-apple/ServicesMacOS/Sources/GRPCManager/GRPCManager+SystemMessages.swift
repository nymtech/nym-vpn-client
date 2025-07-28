import GRPC
import SwiftProtobuf
import SystemMessageModels

extension GRPCManager {
    public func fetchSystemMessages() async throws -> [NymNetworkMessage] {
        let result = try await client.getSystemMessages(Google_Protobuf_Empty())
        return result.messages.map {
            NymNetworkMessage(name: $0.name, message: $0.message, properties: $0.properties)
        }
    }

    public func fetchCompatibleVersions() async throws -> (macOS: String?, core: String?) {
        let result = try await client.getNetworkCompatibility(Google_Protobuf_Empty())
        return (macOS: result.networkCompatibility.macos, core: result.networkCompatibility.core)
    }
}
