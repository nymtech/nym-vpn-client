import Foundation
#if os(iOS)
import NymVPNLib
#endif
import Theme

public enum ErrorReason: LocalizedError, Codable {
#if os(macOS)
    case existingAccount
#endif
    // App
    case offline
    case noAccountStored
    case noDeviceStored
    // Tunnel
    case firewall
    case routing
    case dns
    case internalUnknown
    case sameEntryAndExitGateway
    case invalidEntryGatewayCountry
    case invalidExitGatewayCountry
    case maxDevicesReached
    case bandwidthExceeded
    case api(String)
    case apiTimeout
    case apiStatusCode(String)
    case apiResponse(String)
    case registrationInProgress
    case internalError(String)
    case deviceTimeOutOfSync
    case createMixnetStorage
    case ipv6Unavailable
    case inactiveSubscription
    case accountControl(String)
    case unknown

    private static let somethingWentWrong = "generalNymError.somethingWentWrong".localizedString

    public static let domain = "ErrorHandler.ErrorReason"

#if os(iOS)
    public init(with errorStateReason: ErrorStateReason) {
        switch errorStateReason {
        case .internal:
            self = .internalUnknown
        case .sameEntryAndExitGateway:
            self = .sameEntryAndExitGateway
        case .invalidEntryGatewayCountry:
            self = .invalidEntryGatewayCountry
        case .invalidExitGatewayCountry:
            self = .invalidExitGatewayCountry
        case .maxDevicesReached:
            self = .maxDevicesReached
        case .bandwidthExceeded:
            self = .bandwidthExceeded
        case .deviceTimeOutOfSync:
            self = .deviceTimeOutOfSync
        case .ipv6Unavailable:
            self = .ipv6Unavailable
        case .inactiveSubscription:
            self = .inactiveSubscription
        case .setFirewallPolicy:
            self = .firewall
        case .setRouting:
            self = .routing
        case .setDns:
            self = .dns
        case .tunDevice:
            self = .internalError("tunDevice")
        case .tunnelProvider:
            self = .internalError("tunnelProvider")
        case .badBandwidthIncrease:
            self = .internalError("badBandwidthIncrease")
        case .inactiveAccount:
            self = .internalError("inactiveAccount")
        case .deviceLoggedOut:
            self = .internalError("deviceLoggedOut")
        }
    }
#endif

    public init?(nsError: NSError) {
        guard nsError.domain == ErrorReason.domain,
              let errorReason = ErrorReasonCode(rawValue: nsError.code)
        else {
            self = .unknown
            return
        }

        switch errorReason {
        case .unknown:
            self = .unknown
        case .offline:
            self = .offline
        case .noAccountStored:
            self = .noAccountStored
        case .noDeviceStored:
            self = .noDeviceStored
        case .firewall:
            self = .firewall
        case .routing:
            self = .routing
        case .dns:
            self = .dns
        case .internalUnknown:
            self = .internalUnknown
        case .sameEntryAndExitGateway:
            self = .sameEntryAndExitGateway
        case .invalidEntryGatewayCountry:
            self = .invalidEntryGatewayCountry
        case .invalidExitGatewayCountry:
            self = .invalidExitGatewayCountry
        case .maxDevicesReached:
            self = .maxDevicesReached
        case .bandwidthExceeded:
            self = .bandwidthExceeded
        case .api:
            self = .api(nsError.userInfo["details"] as? String ?? Self.somethingWentWrong)
        case .registrationInProgress:
            self = .registrationInProgress
        case .internalError:
            self = .internalError(nsError.userInfo["details"] as? String ?? Self.somethingWentWrong)
        case .deviceTimeOutOfSync:
            self = .deviceTimeOutOfSync
        case .apiTimeout:
            self = .apiTimeout
        case .apiStatusCode:
            self = .apiStatusCode(nsError.userInfo["details"] as? String ?? Self.somethingWentWrong)
        case .apiResponse:
            self = .apiResponse(nsError.userInfo["details"] as? String ?? Self.somethingWentWrong)
        case .createMixnetStorage:
            self = .createMixnetStorage
        case .ipv6Unavailable:
            self = .ipv6Unavailable
        case .inactiveSubscription:
            self = .inactiveSubscription
        case .accountControl:
            self = .accountControl(nsError.userInfo["details"] as? String ?? Self.somethingWentWrong)
#if os(macOS)
        case .existingAccount:
            self = .existingAccount
#endif
        }
    }

    public var errorDescription: String? {
        description
    }

    public var nsError: NSError {
        let userInfo: [String: String] = [
            "details": description
        ]
        return NSError(
            domain: ErrorReason.domain,
            code: errorCode,
            userInfo: userInfo
        )
    }
}

extension ErrorReason {
    var errorCode: Int {
        ErrorReasonCode(errorReason: self)?.rawValue ?? 0
    }
}

private extension ErrorReason {
    var description: String {
        switch self {
        case .firewall:
            "errorReason.firewall".localizedString
        case .routing:
            "errorReason.routing".localizedString
        case .dns:
            "errorReason.dns".localizedString
        case .internalUnknown:
            "errorReason.internalUnknown".localizedString
        case .sameEntryAndExitGateway:
            "errorReason.sameEntryAndExitGateway".localizedString
        case .invalidEntryGatewayCountry:
            "errorReason.invalidEntryGatewayCountry".localizedString
        case .invalidExitGatewayCountry:
            "errorReason.invalidExitGatewayCountry".localizedString
        case .unknown:
            "errorReason.unknown".localizedString
        case .offline:
            "errorReason.offline".localizedString
        case .noAccountStored:
            "errorReason.noAccountStored".localizedString
        case .noDeviceStored:
            "errorReason.noDeviceStored".localizedString
        case .maxDevicesReached:
            "errorReason.maxDevicesReached".localizedString
        case .bandwidthExceeded:
            "errorReason.bandwidthExceeded".localizedString
        case .inactiveSubscription:
            "errorReason.subscriptionExpired".localizedString
        case let .api(message):
            message
        case .registrationInProgress:
            "errorReason.registrattionInProgress".localizedString
        case let .internalError(message):
            message
        case .deviceTimeOutOfSync:
            "errorReason.deviceTimeOutOfSync".localizedString
        case .apiTimeout:
            "errorReason.apiTimeout".localizedString
        case let .apiStatusCode(code):
            code
        case let .apiResponse(message):
            message
        case .createMixnetStorage:
            "errorReason.createMixnetStorage".localizedString
        case .ipv6Unavailable:
            "errorReason.ipv6Unavailable".localizedString
        case let .accountControl(message):
            message
#if os(macOS)
        case .existingAccount:
            "errorReason.existingAccount".localizedString
#endif
        }
    }
}

extension ErrorReason: Equatable {
    public static func == (lhs: ErrorReason, rhs: ErrorReason) -> Bool {
        lhs.errorCode == rhs.errorCode
    }
}

enum ErrorReasonCode: Int, RawRepresentable {
#if os(macOS)
    case existingAccount
#endif
    case unknown
    case offline
    case noAccountStored
    case noDeviceStored
    case firewall
    case routing
    case dns
    case internalUnknown
    case sameEntryAndExitGateway
    case invalidEntryGatewayCountry
    case invalidExitGatewayCountry
    case maxDevicesReached
    case bandwidthExceeded
    case api
    case apiTimeout
    case apiStatusCode
    case apiResponse
    case internalError
    case registrationInProgress
    case deviceTimeOutOfSync
    case createMixnetStorage
    case ipv6Unavailable
    case inactiveSubscription
    case accountControl

    init?(errorReason: ErrorReason) {
        switch errorReason {
        case .unknown:
            self = .unknown
        case .offline:
            self = .offline
        case .noAccountStored:
            self = .noAccountStored
        case .noDeviceStored:
            self = .noDeviceStored
        case .firewall:
            self = .firewall
        case .routing:
            self = .routing
        case .dns:
            self = .dns
        case .internalUnknown:
            self = .internalUnknown
        case .sameEntryAndExitGateway:
            self = .sameEntryAndExitGateway
        case .invalidEntryGatewayCountry:
            self = .invalidEntryGatewayCountry
        case .invalidExitGatewayCountry:
            self = .invalidExitGatewayCountry
        case .maxDevicesReached:
            self = .maxDevicesReached
        case .bandwidthExceeded:
            self = .bandwidthExceeded
        case .api:
            self = .api
        case .registrationInProgress:
            self = .registrationInProgress
        case .internalError:
            self = .internalError
        case .deviceTimeOutOfSync:
            self = .deviceTimeOutOfSync
        case .apiTimeout:
            self = .apiTimeout
        case .apiStatusCode:
            self = .apiStatusCode
        case .apiResponse:
            self = .apiResponse
        case .createMixnetStorage:
            self = .createMixnetStorage
        case .ipv6Unavailable:
            self = .ipv6Unavailable
        case .inactiveSubscription:
            self = .inactiveSubscription
        case .accountControl:
            self = .accountControl
#if os(macOS)
        case .existingAccount:
            self = .existingAccount
#endif
        }
    }
}
