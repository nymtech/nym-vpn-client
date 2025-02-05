import Foundation
import MixnetLibrary
import Tunnels
import TunnelStatus

extension PacketTunnelProvider {
    override func handleAppMessage(_ messageData: Data) async -> Data? {
        guard let message = try? TunnelProviderMessage(messageData: messageData)
        else {
            return nil
        }
        switch message {
        case .status:
            guard let tunnelState = await tunnelActor.tunnelState else { return nil }
            do {
                return try JSONEncoder().encode(TunnelStatus(from: tunnelState))
            } catch {
                logger.error("AppMessage: \(error.localizedDescription)")
                return nil
            }
        }
    }
}
