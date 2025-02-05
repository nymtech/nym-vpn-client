import GRPC
import SwiftProtobuf

extension GRPCManager {
    public func deviceIdentifier() async throws -> String {
        try await withCheckedThrowingContinuation { continuation in
            let call = client.getDeviceIdentity(Google_Protobuf_Empty())

            call.response.whenComplete { result in
                switch result {
                case .success(let response):
                    continuation.resume(returning: response.deviceIdentity)
                case .failure(let error):
                    continuation.resume(throwing: error)
                }
            }
        }
    }
}
