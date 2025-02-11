import GRPC
import Foundation
import SwiftProtobuf
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
            tunnelStatus = .connected
            connectedDate = Date(timeIntervalSince1970: details.connectionData.connectedAt.timeIntervalSince1970)
        case let .connecting(details):
            tunnelStatus = .connecting
            connectedDate = Date(timeIntervalSince1970: details.connectionData.connectedAt.timeIntervalSince1970)
        case .disconnected:
            tunnelStatus = .disconnected
        case .disconnecting:
            tunnelStatus = .disconnecting
        case let .error(details):
            tunnelStatus = .disconnected
            errorReason = ErrorReason(with: details.reason)
        case let .offline(details):
            tunnelStatus = details.reconnect ? .offlineReconnect : .offline
        case .none:
            tunnelStatus = .unknown
        }
    }
}
