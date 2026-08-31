#if os(iOS)
import Foundation
import NymVPNLib
import Theme

public enum VPNErrorReason: LocalizedError {
    case initialization(details: String)
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
    case requestZknym(details: String)
    case offline
    case unexpectedVpnApiResponse(details: String)
    case failedAccountRegistration(details: String)
    case existingAccount
    case accountControllerError(details: String)
    case httpClient(details: String)
    case accountDoesntExistOnChain
    case accountNotDecentralised
    case accountDecentralised
    case insufficientFunds
    case zkNymAcquisitionFailure(details: String)
    case nyxdConnectionFailure(details: String)
    case nyxdQueryFailure(details: String)
    case invalidSecret(details: String)
    case initLogs(details: String)
    case deeplinkError(details: String)
    case fetchEnvironment(details: String)
    case linkPrivy(details: String)
    case accountStoreBusy
    case unkownTunnelState

    private static let somethingWentWrong = "generalNymError.somethingWentWrong".localizedString
    public static let domain = "ErrorHandler.VPNErrorReason"

    // MARK: - Initializer from VpnError

    // swiftlint:disable:next function_body_length
    public init(with vpnError: VpnError) {
        switch vpnError {
        case let .Initialization(details: details):
            self = .initialization(details: details)
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
            switch vpnApiErrorResponse {
            case .Timeout:
                self = .vpnApiTimeout
            case let .StatusCode(code: code, msg: message):
                self = .vpnApi(details: String("\(code): \(message)"))
            case let .Response(errorResponse):
                self = .vpnApi(details: errorResponse.message)
            }
        case .VpnApiTimeout:
            self = .vpnApiTimeout
        case let .InvalidMnemonic(details: details):
            self = .invalidMnemonic(details: details)
        case let .UnregisterDevice(details: details):
            self = .unregisterDevice(details: details)
        case let .UnexpectedVpnApiResponse(details: details):
            self = .unexpectedVpnApiResponse(details: details)
        case let .FailedAccountRegistration(details: details):
            self = .failedAccountRegistration(details: details)
        case .ExistingAccount:
            self = .existingAccount
        case let .AccountControllerError(details: details):
            self = .accountControllerError(details: details)
        case let .HttpClient(details):
            self = .httpClient(details: details)
        case .AccountDoesntExistOnChain:
            self = .accountDoesntExistOnChain
        case .AccountNotDecentralised:
            self = .accountNotDecentralised
        case .AccountDecentralised:
            self = .accountDecentralised
        case .InsufficientFunds:
            self = .insufficientFunds
        case let .ZkNymAcquisitionFailure(details: details):
            self = .zkNymAcquisitionFailure(details: details)
        case let .NyxdConnectionFailure(details: details):
            self = .nyxdConnectionFailure(details: details)
        case let .NyxdQueryFailure(details: details):
            self = .nyxdQueryFailure(details: details)
        case let .InvalidSecret(details: details):
            self = .invalidSecret(details: details)
        case let .InitLogs(details: details):
            self = .initLogs(details: details)
        case let .DeeplinkError(details: details):
            self = .deeplinkError(details: details)
        case let .FetchEnvironment(details: details):
            self = .fetchEnvironment(details: details)
        case let .LinkPrivyAccount(details: details):
            self = .linkPrivy(details: details)
        }
    }

    // MARK: - Initializer from NSError
    public init?(nsError: NSError) {
        guard nsError.domain == VPNErrorReason.domain,
              let errorReason = VPNErrorReasonCode(rawValue: nsError.code)
        else {
            self = .unkownTunnelState
            return
        }

        switch errorReason {
        case .initialization:
            self = .initialization(details: nsError.userInfo["details"] as? String ?? Self.somethingWentWrong)
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
        case .requestZknym:
            self = .requestZknym(details: nsError.userInfo["details"] as? String ?? Self.somethingWentWrong)
        case .offline:
            self = .offline
        case .unkownTunnelState:
            self = .unkownTunnelState
        case .unexpectedVpnApiResponse:
            self = .unexpectedVpnApiResponse(details: nsError.userInfo["details"] as? String ?? Self.somethingWentWrong)
        case .failedAccountRegistration:
            self = .failedAccountRegistration(
                details: nsError.userInfo["details"] as? String ?? Self.somethingWentWrong
            )
        case .accountControllerError:
            self = .accountControllerError(details: nsError.userInfo["details"] as? String ?? Self.somethingWentWrong)
        case .existingAccount:
            self = .existingAccount
        case .httpClient:
            self = .httpClient(details: nsError.userInfo["details"] as? String ?? Self.somethingWentWrong)
        case .accountDoesntExistOnChain:
            self = .accountDecentralised
        case .accountNotDecentralised:
            self = .accountNotDecentralised
        case .accountDecentralised:
            self = .accountDecentralised
        case .insufficientFunds:
            self = .insufficientFunds
        case .zkNymAcquisitionFailure:
            self = .zkNymAcquisitionFailure(details: nsError.userInfo["details"] as? String ?? Self.somethingWentWrong)
        case .nyxdConnectionFailure:
            self = .nyxdQueryFailure(details: nsError.userInfo["details"] as? String ?? Self.somethingWentWrong)
        case .nyxdQueryFailure:
            self = .nyxdQueryFailure(details: nsError.userInfo["details"] as? String ?? Self.somethingWentWrong)
        case .invalidSecret:
            self = .invalidSecret(details: nsError.userInfo["details"] as? String ?? Self.somethingWentWrong)
        case .initLogs:
            self = .initLogs(details: nsError.userInfo["details"] as? String ?? Self.somethingWentWrong)
        case .deeplinkError:
            self = .deeplinkError(details: nsError.userInfo["details"] as? String ?? Self.somethingWentWrong)
        case .fetchEnvironment:
            self = .fetchEnvironment(details: nsError.userInfo["details"] as? String ?? Self.somethingWentWrong)
        case .linkPrivy:
            self = .linkPrivy(details: nsError.userInfo["details"] as? String ?? Self.somethingWentWrong)
        case .accountStoreBusy:
            self = .accountStoreBusy
        }
    }

    // MARK: - Error Description & NSError Conversion

    public var errorDescription: String? {
        description
    }

    public var nsError: NSError {
        let userInfo: [String: String] = [
            "details": description
        ]
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
        case let .requestZknym(details):
            details
        case .unkownTunnelState:
            "errorReason.unknownTunnelState".localizedString
        case .offline:
            "errorReason.offline".localizedString
        case let .unexpectedVpnApiResponse(details: details):
            details
        case let .failedAccountRegistration(details: details):
            details
        case .existingAccount:
            "errorReason.existingAccount".localizedString
        case let .accountControllerError(details: details):
            details
        case let .httpClient(details: details):
            details
        case .accountDoesntExistOnChain:
            "errorReason.accountDoesntExistOnChain".localizedString
        case .accountNotDecentralised:
            "errorReason.accountNotDecentralised".localizedString
        case .accountDecentralised:
            "errorReason.accountDecentralised".localizedString
        case .insufficientFunds:
            "errorReason.insufficientFunds".localizedString
        case let .zkNymAcquisitionFailure(details: details):
            details
        case let .nyxdConnectionFailure(details: details):
            details
        case let .nyxdQueryFailure(details: details):
            details
        case let .initialization(details: details):
            details
        case let .invalidSecret(details: details):
            details
        case let .initLogs(details: details):
            details
        case let .deeplinkError(details: details):
            details
        case let .fetchEnvironment(details: details):
            details
        case let .linkPrivy(details: details):
            details
        case .accountStoreBusy:
            "errorReason.accountStoreBusy".localizedString
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
    case initialization
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
    case requestZknym
    case offline
    case unkownTunnelState
    case unexpectedVpnApiResponse
    case failedAccountRegistration
    case existingAccount
    case accountControllerError
    case httpClient
    case accountDoesntExistOnChain
    case accountNotDecentralised
    case accountDecentralised
    case insufficientFunds
    case zkNymAcquisitionFailure
    case nyxdConnectionFailure
    case nyxdQueryFailure
    case invalidSecret
    case initLogs
    case deeplinkError
    case fetchEnvironment
    case linkPrivy
    // Appended last to keep existing NSError wire codes stable across the app/extension boundary.
    case accountStoreBusy

    init?(vpnErrorReason: VPNErrorReason) {
        switch vpnErrorReason {
        case .initialization:
            self = .initialization
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
        case .requestZknym:
            self = .requestZknym
        case .unkownTunnelState:
            self = .unkownTunnelState
        case .offline:
            self = .offline
        case .unexpectedVpnApiResponse:
            self = .unexpectedVpnApiResponse
        case .failedAccountRegistration:
            self = .failedAccountRegistration
        case .existingAccount:
            self = .existingAccount
        case .accountControllerError:
            self = .accountControllerError
        case .httpClient:
            self = .httpClient
        case .accountDoesntExistOnChain:
            self = .accountDoesntExistOnChain
        case .accountNotDecentralised:
            self = .accountNotDecentralised
        case .accountDecentralised:
            self = .accountDecentralised
        case .insufficientFunds:
            self = .insufficientFunds
        case .zkNymAcquisitionFailure:
            self = .zkNymAcquisitionFailure
        case .nyxdConnectionFailure:
            self = .nyxdConnectionFailure
        case .nyxdQueryFailure:
            self = .nyxdQueryFailure
        case .invalidSecret:
            self = .invalidSecret
        case .initLogs:
            self = .initLogs
        case .deeplinkError:
            self = .deeplinkError
        case .fetchEnvironment:
            self = .fetchEnvironment
        case .linkPrivy:
            self = .linkPrivy
        case .accountStoreBusy:
            self = .accountStoreBusy
        }
    }
}
#endif
