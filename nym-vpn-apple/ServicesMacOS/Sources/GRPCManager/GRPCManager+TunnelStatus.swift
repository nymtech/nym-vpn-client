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
        case let .connecting(retryAttempt: retryAttempt, state: state, tunnelType: tunnelType, connectionData: connectionData):
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
        switch tunnelStateError {
        case .setFirewallPolicy:
            ErrorReason.setFirewallPolicy
        case .setRouting:
            ErrorReason.setRouting
        case .sameEntryAndExitGateway:
            ErrorReason.sameEntryAndExitGateway
        case .invalidEntryGatewayCountry:
            ErrorReason.invalidEntryGatewayCountry
        case .invalidExitGatewayCountry:
            ErrorReason.invalidExitGatewayCountry
        case .maxDevicesReached:
            ErrorReason.maxDevicesReached
        case .bandwidthExceeded:
            ErrorReason.bandwidthExceeded
        case .setDns:
            ErrorReason.setDns
        case let .internal(code):
            ErrorReason.internalError(code)
        case .deviceTimeOutOfSync:
            ErrorReason.deviceTimeOutOfSync
        case .ipv6Unavailable:
            ErrorReason.ipv6Unavailable
        case .inactiveSubscription:
            ErrorReason.inactiveSubscription
        case .tunDevice:
            ErrorReason.tunDevice
        case .tunnelProvider:
            ErrorReason.tunnelProvider
        case .inactiveAccount:
            ErrorReason.inactiveAccount
        case .deviceLoggedOut:
            ErrorReason.deviceLoggedOut
        case .credentialWastedOnEntryGateway:
            ErrorReason.credentialWastedOnEntryGateway
        case .credentialWastedOnExitGateway:
            ErrorReason.credentialWastedOnExitGateway
        case .performantEntryGatewayUnavailable:
            ErrorReason.performantEntryGatewayUnavailable
        case .performantExitGatewayUnavailable:
            ErrorReason.performantExitGatewayUnavailable
        case .invalidEntryGatewayIdentity:
            ErrorReason.invalidEntryGatewayCountry
        case .invalidExitGatewayIdentity:
            ErrorReason.invalidExitGatewayIdentity
        case .needFullDiskPermissions:
            ErrorReason.needFullDiskPermissions
        case .splitTunnel:
            ErrorReason.splitTunnel
        case .needsRelaxedIndependenceCriteria:
            ErrorReason.needsRelaxedIndependenceCriteria
        case .credentialFetchingFailed:
            ErrorReason.credentialFetchingFailed
        case .noCredentialAvailable:
            ErrorReason.noCredentialAvailable
        }
    }
}

#if os(macOS)
extension ErrorReason {
    init(with tunnelStateError: ErrorStateReason) {
        switch tunnelStateError {
        case .setFirewallPolicy:
            self = .setFirewallPolicy
        case .setRouting:
            self = .setRouting
        case .sameEntryAndExitGateway:
            self = .sameEntryAndExitGateway
        case .invalidEntryGatewayCountry:
            self = .invalidEntryGatewayCountry
        case .setDns:
            self = .setDns
        case .tunDevice:
            self = .tunDevice
        case .tunnelProvider:
            self = .tunnelProvider
        case .ipv6Unavailable:
            self = .ipv6Unavailable
        case .invalidExitGatewayCountry:
            self = .invalidExitGatewayCountry
        case .invalidEntryGatewayIdentity:
            self = .invalidEntryGatewayIdentity
        case .invalidExitGatewayIdentity:
            self = .invalidExitGatewayIdentity
        case .bandwidthExceeded:
            self = .bandwidthExceeded
        case .inactiveSubscription:
            self = .inactiveSubscription
        case .inactiveAccount:
            self = .inactiveAccount
        case .maxDevicesReached:
            self = .maxDevicesReached
        case .deviceTimeOutOfSync:
            self = .deviceTimeOutOfSync
        case .deviceLoggedOut:
            self = .deviceLoggedOut
        case .internal:
            self = .internalUnknown
        case .credentialWastedOnEntryGateway:
            self = .credentialWastedOnEntryGateway
        case .credentialWastedOnExitGateway:
            self = .credentialWastedOnExitGateway
        case .performantEntryGatewayUnavailable:
            self = .performantEntryGatewayUnavailable
        case .performantExitGatewayUnavailable:
            self = .performantExitGatewayUnavailable
        case .needFullDiskPermissions:
            self = .needFullDiskPermissions
        case .splitTunnel:
            self = .splitTunnel
        case .needsRelaxedIndependenceCriteria:
            self = .needsRelaxedIndependenceCriteria
        case .credentialFetchingFailed:
            self = .credentialFetchingFailed
        case .noCredentialAvailable:
            self = .noCredentialAvailable
        }
    }
}
#endif

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
