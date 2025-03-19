import GRPC
import Foundation
import SwiftProtobuf
import Constants
import ErrorReason

extension GRPCManager {
    func setupListenToTunnelStateChangesObserver() {
        let call = client.listenToTunnelState(Google_Protobuf_Empty()) { [weak self] tunnelState in
            self?.updateTunnelStatus(with: tunnelState)
        }

        call.status.whenComplete { result in
            switch result {
            case .success(let status):
                print("Stream completed with status: \(status)")
            case .failure(let error):
                print("Stream failed with error: \(error)")
            }
        }
    }
}

extension GRPCManager {
    func updateTunnelStatus(with state: Nym_Vpn_TunnelState) {
        switch state.state {
        case let .connected(details):
            connectedDate = Date(timeIntervalSince1970: details.connectionData.connectedAt.timeIntervalSince1970)
            tunnelStatus = .connected
        case let .connecting(details):
            connectedDate = Date(timeIntervalSince1970: details.connectionData.connectedAt.timeIntervalSince1970)
            tunnelStatus = .connecting
        case .disconnected:
            tunnelStatus = .disconnected
        case .disconnecting:
            tunnelStatus = .disconnecting
        case let .error(details):
            tunnelStatus = .disconnected
            errorReason = resolveError(with: details)
        case let .offline(details):
            tunnelStatus = details.reconnect ? .offlineReconnect : .offline
        case .none:
            tunnelStatus = .unknown
        }
    }
}

extension GRPCManager {
    func resolveError(with tunnelStateError: Nym_Vpn_TunnelState.Error) -> Error? {
        switch tunnelStateError.reason {
        case .firewall:
            ErrorReason.firewall
        case .routing:
            ErrorReason.routing
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
        case .subscriptionExpired:
            ErrorReason.subscriptionExpired
        case .dns:
            ErrorReason.dns
        case .api:
            ErrorReason.api(tunnelStateError.detail)
        case .internal:
            ErrorReason.internalUnknown
        case .UNRECOGNIZED:
            ErrorReason.unknown
        }
    }
}
