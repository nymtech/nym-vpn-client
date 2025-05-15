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
                let retryAttempt: Int?
                switch tunnelState {
                case let .connecting(retryAttempt: attempt, connectionData: _):
                    retryAttempt = Int(attempt)
                default:
                    retryAttempt = nil
                }

                let statusResponse = TunnelStatusResponse(
                    status: TunnelStatus(from: tunnelState),
                    retryAttempt: retryAttempt
                )
                return try JSONEncoder().encode(statusResponse)
            } catch {
                logger.error("AppMessage: \(error.localizedDescription)")
                return nil
            }
        }
    }
}
