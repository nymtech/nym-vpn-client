#if os(iOS)
import Foundation
import MixnetLibrary

public enum VPNErrorReason: Codable, Error, LocalizedError {
    case internalError(details: String)
    case networkConnectionError(details: String)
    case gatewayError(details: String)
    case invalidCredential(details: String)
    case outOfBandwidth
    case invalidStateError(details: String)
    case accountReady
    case noAccountStored
    case accountNotSynced
    case accountNotRegistered
    case accountNotActive
    case noActiveSubscription
    case accountDeviceNotRegistered
    case accountDeviceNotActive
    case noDeviceIdentity
    case vpnApiTimeout
    case accountUpdateFailed(message: String, messageId: String?)
    case deviceUpdateFailed(message: String, messageId: String?)
    case deviceRegistrationFailed(message: String, messageId: String?)
    case invalidAccountStoragePath(details: String)
    case unkownTunnelState

    public static let domain = "ErrorHandler.VPNErrorReason"

    public init(with vpnError: VpnError) {
        switch vpnError {
        case let .InternalError(details):
            self = .internalError(details: details)
        case let .NetworkConnectionError(details):
            self = .networkConnectionError(details: details)
        case let .GatewayError(details):
            self = .gatewayError(details: details)
        case let .InvalidCredential(details):
            self = .invalidCredential(details: details)
        case .OutOfBandwidth:
            self = .outOfBandwidth
        case let .InvalidStateError(details):
            self = .invalidStateError(details: details)
        case .AccountReady:
            self = .accountReady
        case .NoAccountStored:
            self = .noAccountStored
        case .AccountNotActive:
            self = .accountNotActive
        case .NoActiveSubscription:
            self = .noActiveSubscription
        case .AccountDeviceNotRegistered:
            self = .accountDeviceNotRegistered
        case .AccountDeviceNotActive:
            self = .accountDeviceNotActive
        case .VpnApiTimeout:
            self = .vpnApiTimeout
        case .AccountNotSynced:
            self = .accountNotSynced
        case .AccountNotRegistered:
            self = .accountNotRegistered
        case .NoDeviceIdentity:
            self = .noDeviceIdentity
        case let .AccountUpdateFailed(message: message, messageId: messageId):
            self = .accountUpdateFailed(message: message, messageId: messageId)
        case let .DeviceUpdateFailed(message: message, messageId: messageId):
            self = .deviceUpdateFailed(message: message, messageId: messageId)
        case let .DeviceRegistrationFailed(message: message, messageId: messageId):
            self = .deviceRegistrationFailed(message: message, messageId: messageId)
        case let .InvalidAccountStoragePath(details: details):
            self = .invalidAccountStoragePath(details: details)
        }
    }

    public init?(nsError: NSError) {
        guard nsError.domain == VPNErrorReason.domain else { return nil }
        switch nsError.code {
        case 0:
            self = .internalError(details: nsError.localizedDescription)
        case 1:
            self = .networkConnectionError(details: nsError.localizedDescription)
        case 2:
            self = .gatewayError(details: nsError.localizedDescription)
        case 3:
            self = .invalidCredential(details: nsError.localizedDescription)
        case 4:
            self = .outOfBandwidth
        case 5:
            self = .invalidStateError(details: nsError.localizedDescription)
        case 6:
            self = .accountReady
        case 7:
            self = .noAccountStored
        case 8:
            self = .accountNotSynced
        case 9:
            self = .accountNotRegistered
        case 10:
            self = .accountNotActive
        case 11:
            self = .noActiveSubscription
        case 12:
            self = .accountDeviceNotRegistered
        case 13:
            self = .accountDeviceNotActive
        case 14:
            self = .noDeviceIdentity
        case 15:
            self = .vpnApiTimeout
        case 16:
            self = .accountUpdateFailed(message: nsError.localizedDescription, messageId: nil)
        case 17:
            self = .deviceUpdateFailed(message: nsError.localizedDescription, messageId: nil)
        case 18:
            self = .deviceRegistrationFailed(message: nsError.localizedDescription, messageId: nil)
        case 19:
            self = .invalidAccountStoragePath(details: nsError.localizedDescription)
        default:
            self = .unkownTunnelState
        }
    }

    public var errorDescription: String? {
        description
    }

    public var nsError: NSError {
        NSError(
            domain: VPNErrorReason.domain,
            code: errorCode,
            userInfo: [
                NSLocalizedDescriptionKey: description
            ]
        )
    }
}

private extension VPNErrorReason {
    var errorCode: Int {
        switch self {
        case .internalError:
            return 0
        case .networkConnectionError:
            return 1
        case .gatewayError:
            return 2
        case .invalidCredential:
            return 3
        case .outOfBandwidth:
            return 4
        case .invalidStateError:
            return 5
        case .accountReady:
            return 6
        case .noAccountStored:
            return 7
        case .accountNotSynced:
            return 8
        case .accountNotRegistered:
            return 9
        case .accountNotActive:
            return 10
        case .noActiveSubscription:
            return 11
        case .accountDeviceNotRegistered:
            return 12
        case .accountDeviceNotActive:
            return 13
        case .noDeviceIdentity:
            return 14
        case .vpnApiTimeout:
            return 15
        case .accountUpdateFailed:
            return 16
        case .deviceUpdateFailed:
            return 17
        case .deviceRegistrationFailed:
            return 18
        case .invalidAccountStoragePath:
            return 19
        default:
            return 20
        }
    }

    // TODO: localize
    var description: String {
        switch self {
        case let .internalError(details),
            let .networkConnectionError(details),
            let .gatewayError(details),
            let .invalidCredential(details),
            let .invalidStateError(details),
            let .invalidAccountStoragePath(details: details):
            return details
        case .outOfBandwidth:
            return "The VPN ran out of available bandwidth."
        case .accountReady:
            return "The account is ready."
        case .noAccountStored:
            return "No account information is stored."
        case .accountNotActive:
            return "The account is not active."
        case .noActiveSubscription:
            return "No active subscription found."
        case .accountDeviceNotRegistered:
            return "The device is not registered to the account."
        case .accountDeviceNotActive:
            return "The account device is not active."
        case .vpnApiTimeout:
            return "The VPN API timed out."
        case .accountNotSynced:
            return "The account is not synced."
        case .accountNotRegistered:
            return "The account is not registered."
        case .noDeviceIdentity:
            return "No device identity is available."
        case let .accountUpdateFailed(message: message, messageId: _),
            let .deviceUpdateFailed(message: message, messageId: _),
            let .deviceRegistrationFailed(message: message, messageId: _):
            return message
        case .unkownTunnelState:
            return "Unknown tunnel error reason."
        }
    }
}
#endif
