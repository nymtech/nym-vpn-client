#if os(iOS)
import Foundation
import MixnetLibrary

public enum VPNErrorReason: LocalizedError {
    case internalError(details: String)
    case storage(details: String)
    case networkConnectionError(details: String)
    case invalidStateError(details: String)
    case noAccountStored
    case accountNotRegistered
    case noDeviceIdentity
    case vpnApi(details: String)
    case vpnApiTimeout
    case invalidMnemonic(details: String)
    case invalidAccountStoragePath(details: String)
    case unregisterDevice(details: String)
    case storeAccount(details: String)
    case syncAccount(details: String)
    case syncDevice(details: String)
    case registerDevice(details: String)
    case requestZknym(details: String)
    case requestZkNymBundle(successes: [String], failed: [String])
    case forgetAccount(details: String)
    case unkownTunnelState

    public static let domain = "ErrorHandler.VPNErrorReason"

    // swiftlint:disable:next function_body_length
    public init(with vpnError: VpnError) {
        switch vpnError {
        case let .InternalError(details: details):
            self = .internalError(details: details)
        case let .Storage(details: details):
            self = .storage(details: details)
        case let .NetworkConnectionError(details: details):
            self = .networkConnectionError(details: details)
        case let .InvalidStateError(details: details):
            self = .invalidStateError(details: details)
        case .NoAccountStored:
            self = .noAccountStored
        case .AccountNotRegistered:
            self = .accountNotRegistered
        case .NoDeviceIdentity:
            self = .noDeviceIdentity
        case let .VpnApi(details: vpnApiErrorResponse):
            self = .vpnApi(details: vpnApiErrorResponse.message)
        case .VpnApiTimeout:
            self = .vpnApiTimeout
        case let .InvalidMnemonic(details: details):
            self = .invalidMnemonic(details: details)
        case let .InvalidAccountStoragePath(details: details):
            self = .invalidAccountStoragePath(details: details)
        case let .UnregisterDevice(details: details):
            self = .unregisterDevice(details: details)
        case let .StoreAccount(details: details):
            let messageString: String
            switch details {
            case let .storage(message), let .unexpectedResponse(message):
                messageString = message
            case let .getAccountEndpointFailure(vpnApiErrorResponse):
                messageString = vpnApiErrorResponse.message
            }
            self = .storeAccount(details: messageString)
        case let .SyncAccount(details: details):
            let messageString: String
            switch details {
            case .noAccountStored:
                self = .noAccountStored
                return
            case let .errorResponse(vpnApiErrorResponse):
                messageString = vpnApiErrorResponse.message
            case let .unexpectedResponse(message), let .internal(message):
                messageString = message
            }
            self = .syncAccount(details: messageString)
        case let .SyncDevice(details: details):
            let messageString: String
            switch details {
            case .noAccountStored:
                self = .noAccountStored
                return
            case .noDeviceStored:
                self = .noDeviceIdentity
                return
            case let .errorResponse(vpnApiErrorResponse):
                messageString = vpnApiErrorResponse.message
            case let .unexpectedResponse(message), let .internal(message):
                messageString = message
            }
            self = .syncDevice(details: messageString)
        case let .RegisterDevice(details: details):
            let messageString: String
            switch details {
            case .noAccountStored:
                self = .noAccountStored
                return
            case .noDeviceStored:
                self = .noDeviceIdentity
                return
            case let .errorResponse(vpnApiErrorResponse):
                messageString = vpnApiErrorResponse.message
            case let .unexpectedResponse(message):
                messageString = message
            case let .internal(message):
                messageString = message
            }
            self = .registerDevice(details: messageString)
        case let .RequestZkNym(details: details):
            let messageString: String
            switch details {
            case .noAccountStored:
                self = .noAccountStored
                return
            case .noDeviceStored:
                self = .noDeviceIdentity
                return
            case let .vpnApi(vpnApiErrorResponse):
                messageString = vpnApiErrorResponse.message
            case let .unexpectedVpnApiResponse(message), let .storage(message), let .internal(message):
                messageString = message
            }
            self = .requestZknym(details: messageString)
        case let .RequestZkNymBundle(successes: successes, failed: failed):
            let newFailed = failed.compactMap {
                switch $0 {
                case .noAccountStored:
                    return "No account stored"
                case .noDeviceStored:
                    return "No device stored"
                case let .vpnApi(vpnApiErrorResponse):
                    return vpnApiErrorResponse.message
                case let .unexpectedVpnApiResponse(message), let .storage(message), let .internal(message):
                    return message
                }
            }
            self = .requestZkNymBundle(
                successes: successes.compactMap { $0.id },
                failed: newFailed
            )
        case let .ForgetAccount(details: details):
            let messageString: String
            switch details {
            case .registrationInProgress:
                messageString = "Registration in progress."
            case let .updateDeviceErrorResponse(details):
                messageString = details.message
            case let .unexpectedResponse(message), let .removeAccount(message), let .removeDeviceKeys(message),
                let .resetCredentialStorage(message), let .removeAccountFiles(message), let .initDeviceKeys(message):
                messageString = message
            }
            self = .forgetAccount(details: messageString)
        }
    }

    public init?(nsError: NSError) {
        guard nsError.domain == VPNErrorReason.domain else { return nil }
        switch nsError.code {
        case 0:
            self = .internalError(details: nsError.userInfo["details"] as? String ?? "Something went wrong.")
        case 1:
            self = .networkConnectionError(details: nsError.userInfo["details"] as? String ?? "Something went wrong.")
        case 5:
            self = .invalidStateError(details: nsError.userInfo["details"] as? String ?? "Something went wrong.")
        case 7:
            self = .noAccountStored
        case 9:
            self = .accountNotRegistered
        case 14:
            self = .noDeviceIdentity
        case 15:
            self = .vpnApiTimeout
        case 19:
            self = .invalidAccountStoragePath(details: nsError.localizedDescription)
        case 23:
            self = .invalidMnemonic(details: nsError.userInfo["details"] as? String ?? "Something went wrong.")
        default:
            self = .unkownTunnelState
        }
    }

    public var errorDescription: String? {
        description
    }

    public var nsError: NSError {
        let jsonEncoder = JSONEncoder()

        var userInfo: [String: String] = [
            "details": description
        ]
        let requestZknymDetails = requestZknymDetails
        if let requestZknymDetails,
           !requestZknymDetails.successes.isEmpty,
           let jsonData = try? jsonEncoder.encode(requestZknymDetails.successes),
           let jsonString = String(data: jsonData, encoding: .utf8) {
            userInfo["requestZknymSuccesses"] = jsonString
        }
        if let requestZknymDetails,
           !requestZknymDetails.failures.isEmpty,
           let jsonData = try? jsonEncoder.encode(requestZknymDetails.failures),
           let jsonString = String(data: jsonData, encoding: .utf8) {
            userInfo["requestZknymFailures"] = jsonString
        }
        return NSError(
            domain: VPNErrorReason.domain,
            code: errorCode,
            userInfo: userInfo
        )
    }
}

extension VPNErrorReason: Equatable {
    public static func == (lhs: VPNErrorReason, rhs: VPNErrorReason) -> Bool {
        lhs.errorCode == rhs.errorCode
    }
}

private extension VPNErrorReason {
    var errorCode: Int {
        switch self {
        case .internalError:
            0
        case .storage:
            1
        case .networkConnectionError:
            2
        case .invalidStateError:
            3
        case .noAccountStored:
            4
        case .accountNotRegistered:
            5
        case .noDeviceIdentity:
            6
        case .vpnApi:
            7
        case .vpnApiTimeout:
            8
        case .invalidMnemonic:
            9
        case .invalidAccountStoragePath:
            10
        case .unregisterDevice:
            11
        case .storeAccount:
            12
        case .syncAccount:
            13
        case .syncDevice:
            14
        case .registerDevice:
            15
        case .requestZknym:
            16
        case .requestZkNymBundle:
            17
        case .forgetAccount:
            18
        case .unkownTunnelState:
            19
        }
    }

    // TODO: localize
    var description: String {
        switch self {
        case let.internalError(details: details), let .storage(details: details),
            let .networkConnectionError(details: details), let .invalidStateError(details: details),
            let .vpnApi(details: details), let .invalidMnemonic(details: details),
            let .invalidAccountStoragePath(details: details), let .unregisterDevice(details: details),
            let .storeAccount(details: details), let .syncAccount(details: details),
            let .syncDevice(details: details), let .registerDevice(details: details),
            let .requestZknym(details: details), let .forgetAccount(details: details):
            return details
        case let .requestZkNymBundle(successes: successes, failed: failed):
            let successText = successes.first ?? ""
            let failuresText = failed.first ?? ""
            return "\(successText) \(failuresText)"
        case .noAccountStored:
            return "No account stored. Please add a mnemonic."
        case .accountNotRegistered:
            return "Account not registered. Please retry."
        case .noDeviceIdentity:
            return "No device identity. Please reatry."
        case .vpnApiTimeout:
            return "API timeout. Please retry."
        case .unkownTunnelState:
            return "Unknown tunnel state. Please retry."
        }
    }

    var requestZknymDetails: (successes: [String], failures: [String])? {
        switch self {
        case let .requestZkNymBundle(successes: successes, failed: failed):
            return (successes, failed)
        default:
            return nil
        }
    }
}
#endif
