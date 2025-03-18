import GRPC
import SwiftProtobuf

extension GRPCManager {
    public func deleteLog() async throws {
        try await withCheckedThrowingContinuation { continuation in
            let call = client.deleteLogFile(Google_Protobuf_Empty())

            call.response.whenComplete { result in
                switch result {
                case .success:
                    continuation.resume()
                case let .failure(error):
                    continuation.resume(throwing: error)
                }
            }
        }
    }
}
