import GRPC
import SystemMessageModels

extension GRPCManager {
    public func fetchSystemMessages() async throws -> [NymNetworkMessage] {
        guard helperManager.isHelperAuthorizedAndRunning()
        else {
            throw GRPCError.daemonNotRunning
        }
        logger.log(level: .info, "Checking if stored account")

        return try await withCheckedThrowingContinuation { continuation in
            let call = client.getSystemMessages(Nym_Vpn_GetSystemMessagesRequest())

            call.response.whenComplete { result in
                switch result {
                case .success(let response):
                    let messages = response.messages.map {
                        NymNetworkMessage(name: $0.name, message: $0.message, properties: $0.properties)
                    }
                    continuation.resume(returning: messages)
                case .failure(let error):
                    continuation.resume(throwing: error)
                }
            }
        }
    }

}
