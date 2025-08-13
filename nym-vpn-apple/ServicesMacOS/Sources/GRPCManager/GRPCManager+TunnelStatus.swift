import GRPC
import Foundation
import SwiftProtobuf
import Constants
import ErrorReason

extension GRPCManager {
    func setupListenToTunnelStateChangesObserver() {
        var iterator = client.listenToTunnelState(Google_Protobuf_Empty()).makeAsyncIterator()

        Task {
            do {
                while let tunnelState = try await iterator.next() {
                    await MainActor.run {
                        self.updateTunnelStatus(with: tunnelState)
                    }
                }
                await MainActor.run {
                    resetTunnelStateChangeObserver()
                }
            } catch {
                await MainActor.run {
                    logger.error("Listening to tunnel state failed: \(error)")
                    resetTunnelStateChangeObserver()
                }
            }
        }
    }

    func resetTunnelStateChangeObserver() {
        setup()
        tunnelStatus = .unknown
        isServing = false
    }

    func pingDaemonInitialStatus() {
        guard !isServing else { return }
        Task {
            do {
                try await version()
                let tunnelState = try await client.getTunnelState(Google_Protobuf_Empty())
                Task { @MainActor in
                    updateTunnelStatus(with: tunnelState)
                }
            } catch {
                pingDaemonInitialStatus()
            }
        }
    }
}

extension GRPCManager {
    @MainActor func updateTunnelStatus(with state: NymVpnService_TunnelState) {
        switch state.state {
        case let .connected(details):
            connectedDate = Date(timeIntervalSince1970: details.connectionData.connectedAt.timeIntervalSince1970)
            tunnelStatus = .connected
        case let .connecting(details):
            connectedDate = Date(timeIntervalSince1970: details.connectionData.connectedAt.timeIntervalSince1970)
            connectionRetryAttempt = Int(details.retryAttempt)
            tunnelStatus = .connecting
        case .disconnected:
            tunnelStatus = .disconnected
        case let .disconnecting(details):
            switch details.afterDisconnect {
            case .nothing, .UNRECOGNIZED, .error:
                tunnelStatus = .disconnecting
            case .reconnect:
                tunnelStatus = .connecting
            case .offline:
                tunnelStatus = .offline
            }
            if details.afterDisconnect == .reconnect {
                tunnelStatus = .connecting
            } else {
                tunnelStatus = .disconnecting
            }
        case let .error(details):
            tunnelStatus = .error
            errorReason = resolveError(with: details)
        case let .offline(details):
            tunnelStatus = details.reconnect ? .offlineReconnect : .offline
        case .none:
            tunnelStatus = .unknown
        }

        guard !isServing else { return }
        isServing = true
    }
}

extension GRPCManager {
    func resolveError(with tunnelStateError: NymVpnService_TunnelState.Error) -> Error? {
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
        case .dns:
            ErrorReason.dns
        case .api:
            ErrorReason.api(tunnelStateError.detail)
        case .internal:
            ErrorReason.internalError(tunnelStateError.detail)
        case .UNRECOGNIZED:
            ErrorReason.unknown
        case .deviceTimeOutOfSync:
            ErrorReason.deviceTimeOutOfSync
        case .createMixnetStorage:
            ErrorReason.createMixnetStorage
        case .ipv6Unavailable:
            ErrorReason.ipv6Unavailable
        case .inactiveSubscription:
            ErrorReason.inactiveSubscription
        case .accountControl:
            ErrorReason.accountControl(tunnelStateError.detail)
        }
    }
}
