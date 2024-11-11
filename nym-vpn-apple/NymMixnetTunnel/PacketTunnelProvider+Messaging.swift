import Foundation
import MixnetLibrary
import Tunnels

extension PacketTunnelProvider {
    override func handleAppMessage(_ messageData: Data) async -> Data? {
        guard let message = try? TunnelProviderMessage(messageData: messageData) else { return nil }

        switch message {
        case .lastErrorReason:
            if case let .error(reason) = await tunnelActor.tunnelState {
                do {
                    await tunnelActor.setDidSendLastError(with: true)
                    return try ErrorReason(with: reason).encode()
                } catch {
                    logger.error("Failed to encode error reason: \(error)")
                    return nil
                }
            }
        }
        return nil
    }
}
