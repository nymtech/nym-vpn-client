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
    // PacketTunnelProvider
    case createLogFailed(String)
    // Tunnel
    case setFirewallPolicy
    case setRouting
    case setDns
    case internalUnknown
    case sameEntryAndExitGateway
    case invalidEntryGatewayCountry
    case invalidExitGatewayCountry
    case invalidEntryGatewayIdentity
    case invalidExitGatewayIdentity
    case maxDevicesReached
    case bandwidthExceeded
    case credentialFetchingFailed
    case noCredentialAvailable
    case apiTimeout
    case apiStatusCode(String)
    case apiResponse(String)
    case registrationInProgress
    case internalError(String)
    case deviceTimeOutOfSync
    case ipv6Unavailable
    case inactiveSubscription
    case tunDevice
    case tunnelProvider
    case inactiveAccount
    case deviceLoggedOut
    case credentialWastedOnEntryGateway
    case credentialWastedOnExitGateway
    case performantEntryGatewayUnavailable
    case performantExitGatewayUnavailable
    case needFullDiskPermissions
    case splitTunnel
    case needsRelaxedIndependenceCriteria
    case needsDeviceLocation
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
        case .invalidEntryGatewayIdentity:
            self = .invalidEntryGatewayIdentity
        case .invalidExitGatewayIdentity:
            self = .invalidExitGatewayIdentity
        case .maxDevicesReached:
            self = .maxDevicesReached
        case .bandwidthExceeded:
            self = .bandwidthExceeded
        case .credentialFetchingFailed:
            self = .credentialFetchingFailed
        case .noCredentialAvailable:
            self = .noCredentialAvailable
        case .deviceTimeOutOfSync:
            self = .deviceTimeOutOfSync
        case .ipv6Unavailable:
            self = .ipv6Unavailable
        case .inactiveSubscription:
            self = .inactiveSubscription
        case .setFirewallPolicy:
            self = .setFirewallPolicy
        case .setRouting:
            self = .setRouting
        case .setDns:
            self = .setDns
        case .tunDevice:
            self = .tunDevice
        case .tunnelProvider:
            self = .tunnelProvider
        case .inactiveAccount:
            self = .inactiveAccount
        case .deviceLoggedOut:
            self = .deviceLoggedOut
        case .credentialWastedOnEntryGateway:
            self = .credentialWastedOnEntryGateway
        case .credentialWastedOnExitGateway:
            self = .credentialWastedOnExitGateway
        case .performantEntryGatewayUnavailable:
            self = .performantEntryGatewayUnavailable
        case .performantExitGatewayUnavailable:
            self = .performantExitGatewayUnavailable
        case .needFullDiskPermissions:
            self = .needFullDiskPermissions
        case .splitTunnel:
            self = .splitTunnel
        case .needsRelaxedIndependenceCriteria:
            self = .needsRelaxedIndependenceCriteria
        case .needsDeviceLocation:
            self = .needsDeviceLocation
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
        case .createLogFailed:
            self = .createLogFailed("Unknown")
        case .setFirewallPolicy:
            self = .setFirewallPolicy
        case .setRouting:
            self = .setRouting
        case .setDns:
            self = .setDns
        case .internalUnknown:
            self = .internalUnknown
        case .sameEntryAndExitGateway:
            self = .sameEntryAndExitGateway
        case .invalidEntryGatewayCountry:
            self = .invalidEntryGatewayCountry
        case .invalidExitGatewayCountry:
            self = .invalidExitGatewayCountry
        case .invalidEntryGatewayIdentity:
            self = .invalidEntryGatewayIdentity
        case .invalidExitGatewayIdentity:
            self = .invalidExitGatewayIdentity
        case .maxDevicesReached:
            self = .maxDevicesReached
        case .bandwidthExceeded:
            self = .bandwidthExceeded
        case .credentialFetchingFailed:
            self = .credentialFetchingFailed
        case .noCredentialAvailable:
            self = .noCredentialAvailable
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
        case .ipv6Unavailable:
            self = .ipv6Unavailable
        case .inactiveSubscription:
            self = .inactiveSubscription
        case .tunDevice:
            self = .tunDevice
        case .tunnelProvider:
            self = .tunnelProvider
        case .inactiveAccount:
            self = .inactiveAccount
        case .deviceLoggedOut:
            self = .deviceLoggedOut

        case .credentialWastedOnEntryGateway:
            self = .credentialWastedOnEntryGateway
        case .credentialWastedOnExitGateway:
            self = .credentialWastedOnExitGateway
        case .performantEntryGatewayUnavailable:
            self = .performantEntryGatewayUnavailable
        case .performantExiGatewayUnavailable:
            self = .performantExitGatewayUnavailable
        case .needFullDiskPermissions:
            self = .needFullDiskPermissions
        case .splitTunnel:
            self = .splitTunnel
#if os(macOS)
        case .existingAccount:
            self = .existingAccount
#endif
        case .needsRelaxedIndependenceCriteria:
            self = .needsRelaxedIndependenceCriteria
        case .needsDeviceLocation:
            self = .needsDeviceLocation
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
        case .createLogFailed(let message):
            "errorReason.createLogFailed".localizedString + ": " + message
        case .setFirewallPolicy:
            "errorReason.firewall".localizedString
        case .setRouting:
            "errorReason.routing".localizedString
        case .setDns:
            "errorReason.dns".localizedString
        case .internalUnknown:
            "errorReason.internalUnknown".localizedString
        case .sameEntryAndExitGateway:
            "errorReason.sameEntryAndExitGateway".localizedString
        case .invalidEntryGatewayCountry:
            "errorReason.invalidEntryGatewayCountry".localizedString
        case .invalidExitGatewayCountry:
            "errorReason.invalidExitGatewayCountry".localizedString
        case .invalidEntryGatewayIdentity:
            "errorReason.invalidEntryGatewayIdentity".localizedString
        case .invalidExitGatewayIdentity:
            "errorReason.invalidExitGatewayIdentity".localizedString
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
        case .registrationInProgress:
            "errorReason.registrationInProgress".localizedString
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
        case .ipv6Unavailable:
            "errorReason.ipv6Unavailable".localizedString
        case .tunDevice:
            "errorReason.tunDevice".localizedString
        case .tunnelProvider:
            "errorReason.tunnelProvider".localizedString
        case .inactiveAccount:
            "errorReason.inactiveAccount".localizedString
        case .deviceLoggedOut:
            "errorReason.deviceLoggedOut".localizedString
        case .credentialWastedOnEntryGateway:
            "errorReason.credentialWastedOnEntryGateway".localizedString
        case .credentialWastedOnExitGateway:
            "errorReason.credentialWastedOnExitGateway".localizedString
        case .performantEntryGatewayUnavailable:
            "errorReason.performantEntryGatewayUnavailable".localizedString
        case .performantExitGatewayUnavailable:
            "errorReason.performantExitGatewayUnavailable".localizedString
        case .splitTunnel:
            "errorReason.splitTunnel".localizedString
        case .needFullDiskPermissions:
            "errorReason.needFullDiskPermissions".localizedString
#if os(macOS)
        case .existingAccount:
            "errorReason.existingAccount".localizedString
#endif
        case .needsRelaxedIndependenceCriteria:
            "errorReason.needsRelaxedIndependenceCriteria".localizedString
        case .needsDeviceLocation:
            "errorReason.needsDeviceLocation".localizedString
        case .credentialFetchingFailed:
            "errorReason.credentialFetchingFailed".localizedString
        case .noCredentialAvailable:
            "errorReason.noCredentialAvailable".localizedString
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
    case createLogFailed
    case setFirewallPolicy
    case setRouting
    case setDns
    case internalUnknown
    case sameEntryAndExitGateway
    case invalidEntryGatewayCountry
    case invalidExitGatewayCountry
    case invalidEntryGatewayIdentity
    case invalidExitGatewayIdentity
    case maxDevicesReached
    case bandwidthExceeded
    case credentialFetchingFailed
    case noCredentialAvailable
    case apiTimeout
    case apiStatusCode
    case apiResponse
    case internalError
    case registrationInProgress
    case deviceTimeOutOfSync
    case ipv6Unavailable
    case inactiveSubscription
    case tunDevice
    case tunnelProvider
    case inactiveAccount
    case deviceLoggedOut
    case credentialWastedOnEntryGateway
    case credentialWastedOnExitGateway
    case performantEntryGatewayUnavailable
    case performantExiGatewayUnavailable
    case needFullDiskPermissions
    case splitTunnel
    case needsRelaxedIndependenceCriteria
    case needsDeviceLocation

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
        case .createLogFailed:
            self = .createLogFailed
        case .internalUnknown:
            self = .internalUnknown
        case .sameEntryAndExitGateway:
            self = .sameEntryAndExitGateway
        case .invalidEntryGatewayCountry:
            self = .invalidEntryGatewayCountry
        case .invalidExitGatewayCountry:
            self = .invalidExitGatewayCountry
        case .invalidEntryGatewayIdentity:
            self = .invalidEntryGatewayIdentity
        case .invalidExitGatewayIdentity:
            self = .invalidExitGatewayIdentity
        case .maxDevicesReached:
            self = .maxDevicesReached
        case .bandwidthExceeded:
            self = .bandwidthExceeded
        case .credentialFetchingFailed:
            self = .credentialFetchingFailed
        case .noCredentialAvailable:
            self = .noCredentialAvailable
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
        case .ipv6Unavailable:
            self = .ipv6Unavailable
        case .inactiveSubscription:
            self = .inactiveSubscription
        case .setFirewallPolicy:
            self = .setFirewallPolicy
        case .setRouting:
            self = .setRouting
        case .setDns:
            self = .setDns
        case .tunDevice:
            self = .tunDevice
        case .tunnelProvider:
            self = .tunnelProvider
        case .inactiveAccount:
            self = .inactiveAccount
        case .deviceLoggedOut:
            self = .deviceLoggedOut
        case .credentialWastedOnEntryGateway:
            self = .credentialWastedOnEntryGateway
        case .credentialWastedOnExitGateway:
            self = .credentialWastedOnExitGateway
        case .performantEntryGatewayUnavailable:
            self = .performantEntryGatewayUnavailable
        case .performantExitGatewayUnavailable:
            self = .performantExiGatewayUnavailable
        case .needFullDiskPermissions:
            self = .needFullDiskPermissions
        case .splitTunnel:
            self = .splitTunnel
#if os(macOS)
        case .existingAccount:
            self = .existingAccount
#endif
        case .needsRelaxedIndependenceCriteria:
            self = .needsRelaxedIndependenceCriteria
        case .needsDeviceLocation:
            self = .needsDeviceLocation
        }
    }
}
