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
    // swiftlint:disable:next function_body_length
    func resolveError(with tunnelStateError: Nym_Vpn_TunnelState.Error) -> Error? {
        switch tunnelStateError.errorStateReason {
        case let .baseReason(reason):
            switch reason {
            case .firewall:
                return ErrorReason.firewall
            case .routing:
                return ErrorReason.routing
            case .dns:
                return ErrorReason.dns
            case .tunDevice:
                return ErrorReason.tunDevice
            case .tunnelProvider:
                return ErrorReason.tunnelProvider
            case .sameEntryAndExitGateway:
                return ErrorReason.sameEntryAndExitGateway
            case .invalidEntryGatewayCountry:
                return ErrorReason.invalidEntryGatewayCountry
            case .invalidExitGatewayCountry:
                return ErrorReason.invalidExitGatewayCountry
            case .badBandwidthIncrease:
                return ErrorReason.badBandwidthIncrease
            case .duplicateTunFd:
                return ErrorReason.duplicateTunFd
            case .internal:
                return ErrorReason.internalUnknown
            case .UNRECOGNIZED:
                return ErrorReason.unknown
            case .resolveGatewayAddrs:
                return ErrorReason.resolveGatewayAddrs
            case .startLocalDnsResolver:
                return ErrorReason.startLocalDnsResolver
            }
        case let .syncAccount(reason):
            if reason.noAccountStored {
                return ErrorReason.noAccountStored
            } else {
                return nil
            }
        case let .syncDevice(reason):
            switch reason.errorDetail {
            case let .noAccountStored(isOn):
                return isOn ? ErrorReason.noAccountStored : nil
            case let .noDeviceStored(isOn):
                return isOn ? ErrorReason.noDeviceStored : nil
            case let .errorResponse(details):
                return GeneralNymError.library(message: details.message)
            case let .unexpectedResponse(message), let .internal(message):
                return GeneralNymError.library(message: message)
            case .none:
                return GeneralNymError.somethingWentWrong
            }
        case let .registerDevice(reason):
            switch reason.errorDetail {
            case let .noAccountStored(isOn):
                return isOn ? ErrorReason.noAccountStored : nil
            case let .noDeviceStored(isOn):
                return isOn ? ErrorReason.noDeviceStored : nil
            case let .errorResponse(details):
                return GeneralNymError.library(message: details.message)
            case let .unexpectedResponse(message), let .internal(message):
                return GeneralNymError.library(message: message)
            case .none:
                return GeneralNymError.somethingWentWrong
            }
        case let .requestZkNym(reason):
            switch reason.outcome {
            case let .noAccountStored(isOn):
                return isOn ? ErrorReason.noAccountStored : nil
            case let .noDeviceStored(isOn):
                return isOn ? ErrorReason.noDeviceStored : nil
            case let .vpnApi(response):
                return GeneralNymError.library(message: response.message)
            case let .unexpectedVpnApiResponse(message):
                return GeneralNymError.library(message: message)
            case let .storage(message):
                return GeneralNymError.library(message: message)
            case let .internal(message):
                return GeneralNymError.library(message: message)
            case .none:
                return GeneralNymError.somethingWentWrong
            }
        case let .requestZkNymBundle(reason):
            let firstFailure = reason.failures.first
            switch firstFailure?.outcome {
            case let .noAccountStored(isOn):
                return isOn ? ErrorReason.noAccountStored : nil
            case let .noDeviceStored(isOn):
                return isOn ? ErrorReason.noDeviceStored : nil
            case let .vpnApi(response):
                return GeneralNymError.library(message: response.message)
            case let .unexpectedVpnApiResponse(message):
                return GeneralNymError.library(message: message)
            case let .storage(message):
                return GeneralNymError.library(message: message)
            case let .internal(message):
                return GeneralNymError.library(message: message)
            case .none:
                return GeneralNymError.somethingWentWrong
            }
        default:
            return ErrorReason.unknown
        }
    }
}
