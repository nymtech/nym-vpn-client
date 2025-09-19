import GRPC
import Foundation
import SwiftProtobuf
import Constants
import ErrorReason
import TunnelStatus

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
        guard isServing else { return }
        isServing = false
    }

    func startDaemonInitialStatusPingerIfNeeded() {
        guard versionPingTask == nil || versionPingTask?.isCancelled == true else { return }

        versionPingTask = Task { [weak self] in
            guard let self else { return }
            await self.pingDaemonInitialStatus()
        }
    }

    func stopInitialStatusPinger() {
        versionPingTask?.cancel()
        versionPingTask = nil
    }

    @MainActor func pingDaemonInitialStatus() async {
        var retryCount = 0
        while !isServing {
            do {
                try await version()
                let tunnelState = try await client.getTunnelState(Google_Protobuf_Empty())
                await MainActor.run {
                    updateTunnelStatus(with: tunnelState)
                }
            } catch is CancellationError {
                return
            } catch {
                 logger.debug("pingDaemonInitialStatus error: \(error)")
            }

            if !isServing {
                retryCount += 1
                if retryCount == 2 {
                    daemonVersion = "update"
                }
                do {
                    try await Task.sleep(for: .seconds(5))
                } catch is CancellationError {
                    logger.debug("pingDaemonInitialStatus cancelled during sleep")
                    return
                } catch {
                    logger.debug("Ping Daemon initial status: \(error)")
                }
            }
            if isServing {
                stopInitialStatusPinger()
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
            connectionRetryAttempt = Int(details.retryAttempt)
            tunnelStatus = .connecting
            tunnelConnectingState = TunnelConnectingState(with: details.state)
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
        case .internal:
            ErrorReason(with: tunnelStateError.reason)
        case .UNRECOGNIZED:
            ErrorReason.unknown
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
        }
    }
}

#if os(macOS)
extension ErrorReason {
    init(with tunnelStateError: NymVpnService_TunnelState.ErrorStateReason) {
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
        case .UNRECOGNIZED:
            self = .internalUnknown
        case .credentialWastedOnEntryGateway:
            self = .credentialWastedOnEntryGateway
        case .credentialWastedOnExitGateway:
            self = .credentialWastedOnExitGateway
        }
    }
}
#endif

private extension TunnelConnectingState {
    init(with state: NymVpnService_EstablishConnectionState) {
        switch state {
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
        case .UNRECOGNIZED(_):
            self = .unrecognized
        }
    }
}
