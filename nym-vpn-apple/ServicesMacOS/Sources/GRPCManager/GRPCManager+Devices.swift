import GRPC
import SwiftProtobuf

extension GRPCManager {
    public func deviceIdentifier() async throws -> String {
        try await client.getDeviceIdentity(Google_Protobuf_Empty()).deviceIdentity
    }

    public func accountIdentifier() async throws -> String {
        try await client.getAccountIdentity(Google_Protobuf_Empty()).accountIdentity
    }
}
