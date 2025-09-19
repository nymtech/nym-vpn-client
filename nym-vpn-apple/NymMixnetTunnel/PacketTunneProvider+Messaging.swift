// swiftlint:disable:next file_name
import Foundation
import NymVPNLib
import Tunnels
import TunnelStatus

extension PacketTunnelProvider {
    // swiftlint:disable:next function_body_length
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
                var tunnelConnectingState: TunnelConnectingState?
                var connectionInfoData: ConnectionInfoData?

                switch tunnelState {
                case let .connecting(
                    retryAttempt: attempt,
                    state: establishConnectionState,
                    tunnelType: _,
                    connectionData: connectionData
                ):
                    retryAttempt = Int(attempt)
                    tunnelConnectingState = TunnelConnectingState(with: establishConnectionState)
                    connectionInfoData = ConnectionInfoData(
                        entryGatewayId: connectionData?.entryGateway.id,
                        exitGatewayId: connectionData?.exitGateway.id
                    )
                case let .connected(connectionData: connectionData):
                    connectionInfoData = ConnectionInfoData(
                        entryGatewayId: connectionData.entryGateway.id,
                        exitGatewayId: connectionData.exitGateway.id
                    )
                case let .disconnecting(afterDisconnect: action):
                    afterDisconnectAction = AfterDisconnectAction.convert(from: action)
                    connectionInfoData = nil
                default:
                    retryAttempt = nil
                    afterDisconnectAction = nil
                    tunnelConnectingState = nil
                    connectionInfoData = nil
                }

                let statusResponse = await TunnelStatusResponse(
                    status: TunnelStatus(from: tunnelState),
                    retryAttempt: retryAttempt,
                    afterDisconnectAction: afterDisconnectAction,
                    lastError: tunnelActor.lastError,
                    tunnelConnectingState: tunnelConnectingState,
                    connectionInfoData: connectionInfoData
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

private extension TunnelConnectingState {
    init(with establishConnectionState: EstablishConnectionState) {
        switch establishConnectionState {
        case .resolvingApiAddresses:
            self = .resolvingApiAddresses
        case .awaitingAccountReadiness:
            self = .awaitingAccountReadiness
        case .refreshingGateways:
            self = .refreshingGateways
        case .selectingGateways:
            self = .selectingGateways
        case .connectingMixnetClient:
            self = .connectingMixnetClient
        case .connectingTunnel:
            self = .connectingTunnel
        }
    }
}
