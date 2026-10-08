import Foundation
import NymVPNLib
import Constants
import ErrorReason
import TunnelStatus

extension GRPCManager {
    func updateTunnelStatus(with state: TunnelState) {
        switch state {
        case let .connected(details):
            connectedDate = Date(timeIntervalSince1970: Double(details.connectedAt))
            tunnelStatus = .connected
            connectionInfoData = ConnectionInfoData(
                entryGatewayId: details.entryGateway.id,
                exitGatewayId: details.exitGateway.id,
                tunnelType: ConnectionTunnelType(details.tunnel)
            )
        case let .connecting(
            retryAttempt: retryAttempt,
            state: state,
            selectorFallbackState: _, // todo: must use this field
            tunnelType: tunnelType,
            connectionData: connectionData
        ):
            connectionRetryAttempt = Int(retryAttempt)
            tunnelStatus = .connecting
            tunnelConnectingState = TunnelConnectingState(with: state)
            connectionInfoData = ConnectionInfoData(
                entryGatewayId: connectionData?.entryGateway.id,
                exitGatewayId: connectionData?.exitGateway.id,
                tunnelType: ConnectionTunnelType(tunnelType)
            )
        case .disconnected:
            tunnelStatus = .disconnected
            connectionInfoData = nil
        case let .disconnecting(afterDisconnect):
            switch afterDisconnect {
            case .nothing, .error:
                tunnelStatus = .disconnecting
            case .reconnect:
                tunnelStatus = .connecting
            case .offline:
                tunnelStatus = .offline
            }
            connectionInfoData = nil
        case let .error(details):
            tunnelStatus = .error
            errorReason = resolveError(with: details)
        case let .offline(reconnect: reconnect):
            if reconnect {
                tunnelStatus = .offlineReconnect
            } else {
                tunnelStatus = .offline
            }
        }
    }
}

extension GRPCManager {
    func resolveError(with tunnelStateError: ErrorStateReason) -> Error? {
        if case let .internal(details) = tunnelStateError {
            return ErrorReason(.internalError, details: details)
        }
        return ErrorReason(with: tunnelStateError)
    }
}

private extension ConnectionTunnelType {
    init(_ tunnelType: NymVPNLib.TunnelType) {
        switch tunnelType {
        case .mixnet:
            self = .mixnet
        case .wireguard:
            self = .wireguard
        }
    }

    init(_ data: NymVPNLib.TunnelConnectionData) {
        switch data {
        case .mixnet:
            self = .mixnet
        case .wireguard:
            self = .wireguard
        }
    }
}

private extension TunnelConnectingState {
    init(with state: EstablishConnectionState) {
        switch state {
        case .resolvingApiAddresses:
            self = .resolvingApiAddresses
        case .awaitingAccountReadiness:
            self = .awaitingAccountReadiness
        case .awaitingCredentialsAvailability:
            self = .awaitingCredentialsAvailability
        case .refreshingGateways:
            self = .refreshingGateways
        case .selectingGateways:
            self = .selectingGateways
        case .registeringWithGateways:
            self = .registeringWithGateways
        case .connectingTunnel:
            self = .connectingTunnel
        }
    }
}
