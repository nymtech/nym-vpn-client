import GRPC
import SwiftProtobuf

extension GRPCManager {
    public func deviceIdentifier() async throws -> String {
        try await withCheckedThrowingContinuation { continuation in
            let call = client.getDeviceIdentity(Google_Protobuf_Empty())

            call.response.whenComplete { result in
                switch result {
                case let .success(response):
                    continuation.resume(returning: response.deviceIdentity)
                case let .failure(error):
                    continuation.resume(throwing: error)
                }
            }
        }
    }

    public func accountIdentifier() async throws -> String {
        try await withCheckedThrowingContinuation { continuation in
            let call = client.getAccountIdentity(Google_Protobuf_Empty())
            
            call.response.whenComplete { result in
                switch result {
                case let .success(response):
                    continuation.resume(returning: response.accountIdentity.accountIdentity)
                case let .failure(error):
                    continuation.resume(throwing: error)
                }
            }
        }
    }
}
