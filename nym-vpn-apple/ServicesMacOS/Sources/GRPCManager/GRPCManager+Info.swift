import GRPC
import SwiftProtobuf
import Shell

extension GRPCManager {
    public func version() async throws {
        do {
            let result = try await client.info(
                Google_Protobuf_Empty(),
                callOptions: CallOptions(timeLimit: .timeout(.seconds(7)))
            )
            daemonVersion = result.version
            networkName = result.nymNetwork.networkName
            logger.info("🛜 \(result.nymNetwork.networkName)")
        } catch {
            daemonVersion = "noVersion"
            throw error
        }
    }
}
