import SwiftProtobuf

extension GRPCManager {
    public func switchEnvironment(to environment: String) async throws {
        logger.info("Changing env to \(environment)")

        let request = Google_Protobuf_StringValue(environment)
        _ = try await client.setNetwork(request)
    }
}
