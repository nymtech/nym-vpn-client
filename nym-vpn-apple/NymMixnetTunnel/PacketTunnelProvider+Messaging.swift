import Foundation
import MixnetLibrary
import Tunnels

extension PacketTunnelProvider {
    override func handleAppMessage(_ messageData: Data) async -> Data? {
        guard let message = try? TunnelProviderMessage(messageData: messageData) else { return nil }

        switch message {
        case .lastErrorReason:
            guard let lastErrorStateReason else { return nil }
            do {
                let reason = try ErrorReason(with: lastErrorStateReason).encode()
                didSendError = true
                return reason
            } catch {
                logger.error("Failed to encode error reason: \(error)")
                return nil
            }
        }
    }
}
