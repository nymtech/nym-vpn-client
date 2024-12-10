import GRPC

extension GRPCManager {
    public func deviceIdentifier() async throws -> String {
        try await withCheckedThrowingContinuation { continuation in
            let call = client.getDeviceIdentity(Nym_Vpn_GetDeviceIdentityRequest())

            call.response.whenComplete { result in
                switch result {
                case .success(let response):
                    print(response)
                    continuation.resume(returning: response.deviceIdentity)
                case .failure(let error):
                    continuation.resume(throwing: error)
                }
            }
        }
    }
}
