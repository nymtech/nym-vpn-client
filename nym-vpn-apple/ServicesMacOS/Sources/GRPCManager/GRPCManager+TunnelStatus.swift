import Foundation
import NymVPNRpc
import Constants
import ErrorReason
import TunnelStatus

extension GRPCManager {
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
                guard let tunnelState = try await rpcClient?.getTunnelState() else { return }
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
    @MainActor func updateTunnelStatus(with state: TunnelState) {
        switch state {
        case let .connected(details):
            if let connectedAt = details.connectedAt {
                connectedDate = Date(timeIntervalSince1970: Double(connectedAt))
            }
            tunnelStatus = .connected
            connectionInfoData = ConnectionInfoData(
                entryGatewayId: details.connectionData.entryGateway.id,
                exitGatewayId: details.connectionData.exitGateway.id
            )
        case let .connecting(details):
            connectionRetryAttempt = Int(details.retryAttempt)
            tunnelStatus = .connecting
            tunnelConnectingState = TunnelConnectingState(with: details.state)
            connectionInfoData = ConnectionInfoData(
                entryGatewayId: details.connectionData.entryGateway.id,
                exitGatewayId: details.connectionData.exitGateway.id
            )
        case .disconnected:
            tunnelStatus = .disconnected
            connectionInfoData = nil
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
            connectionInfoData = nil
        case let .error(details):
            tunnelStatus = .error
            errorReason = resolveError(with: errorStateReason)
        case let .offline(reconnect: reconnect):
            if reconnect {
                tunnelStatus = .offlineReconnect
            } else {
                tunnelStatus = .offline
            }
        }

        guard !isServing else { return }
        isServing = true
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
