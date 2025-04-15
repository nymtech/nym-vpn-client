import GRPC
import SwiftProtobuf
import Shell

extension GRPCManager {
    public func version() async throws {
        logger.log(level: .info, "Version")
        return try await withCheckedThrowingContinuation { continuation in
            let call = client.info(
                Google_Protobuf_Empty(),
                callOptions: CallOptions(timeLimit: .timeout(.seconds(5)))
            )

            call.response.whenComplete { [weak self] result in
                switch result {
                case let .success(response):
                    self?.daemonVersion = response.version
                    self?.networkName = response.nymNetwork.networkName
                    self?.logger.info("🛜 \(response.nymNetwork.networkName)")
                    continuation.resume()
                case let .failure(error):
                    self?.daemonVersion = "noVersion"
                    continuation.resume(throwing: error)
                }
            }
        }
    }
}
