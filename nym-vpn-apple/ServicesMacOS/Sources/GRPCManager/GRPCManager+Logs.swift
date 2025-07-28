import GRPC
import SwiftProtobuf

extension GRPCManager {
    public func deleteLog() async throws {
        _ = try await client.deleteLogFile(Google_Protobuf_Empty())
    }
}
