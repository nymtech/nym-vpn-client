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

    public func updateErrorReportingIfNeeded(with isEnabled: Bool) async throws -> Void {
        let isSentryEnabled = try await client.isSentryEnabled(Google_Protobuf_Empty()).value
        guard isSentryEnabled != isEnabled else { return }
        if isEnabled {
            _ = try await client.enableSentry(Google_Protobuf_Empty())
        } else {
            _ = try await client.disableSentry(Google_Protobuf_Empty())
        }
    }
}
