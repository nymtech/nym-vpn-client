import Foundation
import ConnectionTypes
import NymVPNLib
import TunnelMixnet
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
            return await handleStatusMessage()
        case let .setCustomDns(addrs):
            await runCommand { try await self.commandSender?.setCustomDns(addrs: addrs) }
            return nil
        case let .setEnableCustomDns(enabled):
            await runCommand { try await self.commandSender?.setEnableCustomDns(enableCustomDns: enabled) }
            return nil
        case let .setEnableTwoHop(enabled):
            await runCommand { try await self.commandSender?.setEnableTwoHop(enableTwoHop: enabled) }
            return nil
        case let .setEnableAdBlocking(enabled):
            await runCommand { try await self.commandSender?.setEnableAdBlocking(enableAdBlocking: enabled) }
            return nil
        case let .setEnableBridges(enabled):
            await runCommand { try await self.commandSender?.setEnableBridges(enableBridges: enabled) }
            return nil
        case let .setEntryPoint(entry):
            await runCommand { try await self.commandSender?.setEntryPoint(entryPoint: entry.entryPoint) }
            return nil
        case let .setExitPoint(exit):
            await runCommand { try await self.commandSender?.setExitPoint(exitPoint: exit.exitPoint) }
            return nil
        case let .setGatewaySelectionAlgorithm(algorithm):
            await runCommand {
                try await self.commandSender?.setGatewaySelectionAlgorithm(
                    gatewaySelectionAlgorithm: algorithm.sdkValue
                )
            }
            return nil
        case let .setFrontingModeEnabled(enabled):
            await runCommand {
                try await self.commandSender?.setFrontingMode(frontingMode: enabled ? .always : .onRetry)
            }
            return nil
        case let .setDisableIpv6(disabled):
            await runCommand { try await self.commandSender?.setDisableIpv6(disableIpv6: disabled) }
            return nil
        case let .setMixnetTrafficConfig(config):
            await runCommand {
                try await self.commandSender?.setMixnetTrafficConfig(
                    mixnetTrafficConfig: config.mixnetTrafficConfig()
                )
            }
            return nil
        case let .setGatewayIndependence(isEnabled):
            await runCommand {
                try await self.commandSender?.setEnableGatewayIndependence(enableGatewayIndependence: isEnabled)
            }
            if !isEnabled {
                // handle_set_enable_gateway_independence in vpn service
                // calls self.update_tunnel_settings_with_throttle();
                // Throttle == 1 sec
                try? await Task.sleep(for: .seconds(1.2))

                // Suspend path: pre-flight is waiting → resume it (startTunnel
                // then proceeds to connectTunnel).
                let resumedPreflight = await tunnelActor.resumeRelaxConsent()
                if !resumedPreflight {
                    await tunnelActor.clearError()
                    await runCommand {
                        _ = try await self.commandSender?.connectTunnel()
                    }
                }
            }
            return nil
        case let .setGatewayIndependenceNotifications(enabled):
            await runCommand {
                try await self.commandSender?.setGatewayIndependenceNotifications(enableNotifications: enabled)
            }
            return nil
        }
    }
}

private extension PacketTunnelProvider {
    func runCommand(_ block: @escaping () async throws -> Void) async {
        do {
            try await block()
        } catch {
            logger.error("Tunnel command failed: \(error.localizedDescription)")
        }
    }

    // swiftlint:disable:next function_body_length
    func handleStatusMessage() async -> Data? {
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
                tunnelType: tunnelType,
                connectionData: connectionData
            ):
                retryAttempt = Int(attempt)
                tunnelConnectingState = TunnelConnectingState(with: establishConnectionState)
                connectionInfoData = ConnectionInfoData(
                    entryGatewayId: connectionData?.entryGateway.id,
                    exitGatewayId: connectionData?.exitGateway.id,
                    tunnelType: ConnectionTunnelType(tunnelType)
                )
            case let .connected(connectionData: connectionData):
                connectionInfoData = ConnectionInfoData(
                    entryGatewayId: connectionData.entryGateway.id,
                    exitGatewayId: connectionData.exitGateway.id,
                    tunnelType: ConnectionTunnelType(connectionData.tunnel)
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

private extension ConnectionTunnelType {
    init(_ tunnelType: NymVPNLib.TunnelType) {
        switch tunnelType {
        case .mixnet:
            self = .mixnet
        case .wireguard:
            self = .wireguard
        }
    }

    init(_ data: TunnelConnectionData) {
        switch data {
        case .mixnet:
            self = .mixnet
        case .wireguard:
            self = .wireguard
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
        case .awaitingCredentialsAvailability:
            self = .awaitingCredentialsAvailability
        case .refreshingGateways:
            self = .refreshingGateways
        case .selectingGateways:
            self = .selectingGateways
        case .connectingTunnel:
            self = .connectingTunnel
        case .registeringWithGateways:
            self = .registeringWithGateways
        }
    }
}
