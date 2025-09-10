import Foundation
import NymVPNLib
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
                var retryAttempt: Int?
                var afterDisconnectAction: AfterDisconnectAction?

                switch tunnelState {
                case let .connecting(
                    retryAttempt: attempt,
                    state: establishConnectionState,
                    tunnelType: tunnelType,
                    connectionData: _
                ):
                    retryAttempt = Int(attempt)
                case let .disconnecting(afterDisconnect: action):
                    afterDisconnectAction = AfterDisconnectAction.convert(from: action)
                default:
                    retryAttempt = nil
                    afterDisconnectAction = nil
                }

                let statusResponse = await TunnelStatusResponse(
                    status: TunnelStatus(from: tunnelState),
                    retryAttempt: retryAttempt,
                    afterDisconnectAction: afterDisconnectAction,
                    lastError: tunnelActor.lastError
                )
                return try JSONEncoder().encode(statusResponse)
            } catch {
                logger.error("AppMessage: \(error.localizedDescription)")
                return nil
            }
        }
    }
}

private extension AfterDisconnectAction {
    static func convert(from action: ActionAfterDisconnect) -> AfterDisconnectAction? {
        switch action {
        case .nothing, .error:
            nil
        case .reconnect:
            .reconnect
        case .offline:
            .offline
        }
    }
}
