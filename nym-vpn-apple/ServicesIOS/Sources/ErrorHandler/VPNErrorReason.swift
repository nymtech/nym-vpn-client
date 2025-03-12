#if os(iOS)
import Foundation
import MixnetLibrary
import Theme

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

    private static let somethingWentWrong = "generalNymError.somethingWentWrong".localizedString

    public static let domain = "ErrorHandler.VPNErrorReason"

    // MARK: - Initializer from VpnError

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
                    return "errorReason.noAccountStored".localizedString
                case .noDeviceStored:
                    return "errorReason.noDeviceStored".localizedString
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
                messageString = "errorReason.registrationInProgress".localizedString
            case let .updateDeviceErrorResponse(details):
                messageString = details.message
            case let .unexpectedResponse(message),
                let .removeAccount(message), let .removeDeviceKeys(message),
                let .resetCredentialStorage(message), let .removeAccountFiles(message),
                let .initDeviceKeys(message):
                messageString = message
            }
            self = .forgetAccount(details: messageString)
        }
    }

    // MARK: - Initializer from NSError
    // swiftlint:disable:next function_body_length
    public init?(nsError: NSError) {
        guard nsError.domain == VPNErrorReason.domain,
              let errorCode = VPNErrorReasonCode(rawValue: nsError.code)
        else {
            return nil
        }
        switch errorCode {
        case .internalError:
            self = .internalError(details: nsError.userInfo["details"] as? String ?? Self.somethingWentWrong)
        case .storage:
            self = .storage(details: nsError.userInfo["details"] as? String ?? Self.somethingWentWrong)
        case .networkConnectionError:
            self = .networkConnectionError(details: nsError.userInfo["details"] as? String ?? Self.somethingWentWrong)
        case .invalidStateError:
            self = .invalidStateError(details: nsError.userInfo["details"] as? String ?? Self.somethingWentWrong)
        case .noAccountStored:
            self = .noAccountStored
        case .accountNotRegistered:
            self = .accountNotRegistered
        case .noDeviceIdentity:
            self = .noDeviceIdentity
        case .vpnApi:
            self = .vpnApi(details: nsError.userInfo["details"] as? String ?? Self.somethingWentWrong)
        case .vpnApiTimeout:
            self = .vpnApiTimeout
        case .invalidMnemonic:
            self = .invalidMnemonic(details: nsError.userInfo["details"] as? String ?? Self.somethingWentWrong)
        case .invalidAccountStoragePath:
            self = .invalidAccountStoragePath(details: nsError.localizedDescription)
        case .unregisterDevice:
            self = .unregisterDevice(details: nsError.userInfo["details"] as? String ?? Self.somethingWentWrong)
        case .storeAccount:
            self = .storeAccount(details: nsError.userInfo["details"] as? String ?? Self.somethingWentWrong)
        case .syncAccount:
            self = .syncAccount(details: nsError.userInfo["details"] as? String ?? Self.somethingWentWrong)
        case .syncDevice:
            self = .syncDevice(details: nsError.userInfo["details"] as? String ?? Self.somethingWentWrong)
        case .registerDevice:
            self = .registerDevice(details: nsError.userInfo["details"] as? String ?? Self.somethingWentWrong)
        case .requestZknym:
            self = .requestZknym(details: nsError.userInfo["details"] as? String ?? Self.somethingWentWrong)
        case .requestZkNymBundle:
            let decoder = JSONDecoder()
            var successes = [String]()
            var failures = [String]()
            if let successesString = nsError.userInfo["requestZknymSuccesses"] as? String,
               let jsonData = successesString.data(using: .utf8),
               let decodedSuccesses = try? decoder.decode([String].self, from: jsonData) {
                successes = decodedSuccesses
            }
            if let failuresString = nsError.userInfo["requestZknymFailures"] as? String,
               let jsonData = failuresString.data(using: .utf8),
               let decodedFailures = try? decoder.decode([String].self, from: jsonData) {
                failures = decodedFailures
            }
            self = .requestZkNymBundle(successes: successes, failed: failures)
        case .forgetAccount:
            self = .forgetAccount(details: nsError.userInfo["details"] as? String ?? Self.somethingWentWrong)
        case .unkownTunnelState:
            self = .unkownTunnelState
        }
    }

    // MARK: - Error Description & NSError Conversion

    public var errorDescription: String? {
        description
    }

    public var nsError: NSError {
        let jsonEncoder = JSONEncoder()
        var userInfo: [String: String] = [
            "details": description
        ]
        if let requestZknymDetails = requestZknymDetails,
           !requestZknymDetails.successes.isEmpty,
           let jsonData = try? jsonEncoder.encode(requestZknymDetails.successes),
           let jsonString = String(data: jsonData, encoding: .utf8) {
            userInfo["requestZknymSuccesses"] = jsonString
        }
        if let requestZknymDetails = requestZknymDetails,
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

extension VPNErrorReason {
    var errorCode: Int {
        VPNErrorReasonCode(vpnErrorReason: self)?.rawValue ?? 0
    }

    var description: String {
        switch self {
        case let .internalError(details):
            details
        case let .storage(details):
            details
        case let .networkConnectionError(details):
            details
        case let .invalidStateError(details):
            details
        case .noAccountStored:
            "errorReason.noAccountStored".localizedString
        case .accountNotRegistered:
            "errorReason.accountNotRegistered".localizedString
        case .noDeviceIdentity:
            "errorReason.noDeviceStored".localizedString
        case let .vpnApi(details):
            details
        case .vpnApiTimeout:
            "error.timeout".localizedString
        case let .invalidMnemonic(details):
            details
        case let .invalidAccountStoragePath(details):
            details
        case let .unregisterDevice(details):
            details
        case let .storeAccount(details):
            details
        case let .syncAccount(details):
            details
        case let .syncDevice(details):
            details
        case let .registerDevice(details):
            details
        case let .requestZknym(details):
            details
        case let .requestZkNymBundle(successes, failed):
            "\(successes.first ?? "") \(failed.first ?? "")"
        case let .forgetAccount(details):
            details
        case .unkownTunnelState:
            "".localizedString
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

extension VPNErrorReason: Equatable {
    public static func == (lhs: VPNErrorReason, rhs: VPNErrorReason) -> Bool {
        lhs.errorCode == rhs.errorCode
    }
}

/// The VPNErrorReasonCode mirrors the error codes as raw integers and can be constructed from a VPNErrorReason.
enum VPNErrorReasonCode: Int, RawRepresentable {
    case internalError
    case storage
    case networkConnectionError
    case invalidStateError
    case noAccountStored
    case accountNotRegistered
    case noDeviceIdentity
    case vpnApi
    case vpnApiTimeout
    case invalidMnemonic
    case invalidAccountStoragePath
    case unregisterDevice
    case storeAccount
    case syncAccount
    case syncDevice
    case registerDevice
    case requestZknym
    case requestZkNymBundle
    case forgetAccount
    case unkownTunnelState

    init?(vpnErrorReason: VPNErrorReason) {
        switch vpnErrorReason {
        case .internalError:
            self = .internalError
        case .storage:
            self = .storage
        case .networkConnectionError:
            self = .networkConnectionError
        case .invalidStateError:
            self = .invalidStateError
        case .noAccountStored:
            self = .noAccountStored
        case .accountNotRegistered:
            self = .accountNotRegistered
        case .noDeviceIdentity:
            self = .noDeviceIdentity
        case .vpnApi:
            self = .vpnApi
        case .vpnApiTimeout:
            self = .vpnApiTimeout
        case .invalidMnemonic:
            self = .invalidMnemonic
        case .invalidAccountStoragePath:
            self = .invalidAccountStoragePath
        case .unregisterDevice:
            self = .unregisterDevice
        case .storeAccount:
            self = .storeAccount
        case .syncAccount:
            self = .syncAccount
        case .syncDevice:
            self = .syncDevice
        case .registerDevice:
            self = .registerDevice
        case .requestZknym:
            self = .requestZknym
        case .requestZkNymBundle:
            self = .requestZkNymBundle
        case .forgetAccount:
            self = .forgetAccount
        case .unkownTunnelState:
            self = .unkownTunnelState
        }
    }
}
#endif
