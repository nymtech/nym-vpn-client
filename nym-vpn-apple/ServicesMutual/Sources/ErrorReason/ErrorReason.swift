import Foundation
import NymVPNLib
import Theme

public struct ErrorReason: LocalizedError, Codable {
    public enum Code: Int, Codable {
#if os(macOS)
        case existingAccount
#endif
        // App
        case unknown
        case offline
        case noAccountStored
        case noDeviceStored
        // PacketTunnelProvider
        case createLogFailed
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
        case apiStatusCode
        case apiResponse
        case internalError
        case registrationInProgress
        case ipv6Unavailable
        case inactiveSubscription
        case tunDevice
        case tunnelProvider
        case setLocalDnsResolverConfig
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
        case connectionAttemptsExceeded
    }

    public let code: Code
    public let details: String?

    public static let domain = "ErrorHandler.ErrorReason"

    public init(_ code: Code, details: String? = nil) {
        self.code = code
        self.details = details
    }

    public init?(nsError: NSError) {
        guard nsError.domain == ErrorReason.domain,
              let code = Code(rawValue: nsError.code)
        else {
            return nil
        }
        self.init(code, details: nsError.userInfo["details"] as? String)
    }

    public init(with errorStateReason: ErrorStateReason) {
        self.init(Code(with: errorStateReason))
    }

    public var errorDescription: String? {
        switch code {
        case .createLogFailed:
            code.localizedString + ": " + (details ?? "Unknown")
        case .internalError, .apiStatusCode, .apiResponse:
            details ?? "generalNymError.somethingWentWrong".localizedString
        default:
            code.localizedString
        }
    }

    public var nsError: NSError {
        var userInfo: [String: String] = [:]
        if let details {
            userInfo["details"] = details
        }
        return NSError(
            domain: ErrorReason.domain,
            code: code.rawValue,
            userInfo: userInfo
        )
    }
}

extension ErrorReason: Equatable {
    public static func == (lhs: ErrorReason, rhs: ErrorReason) -> Bool {
        lhs.code == rhs.code
    }
}

private extension ErrorReason.Code {
    var localizedString: String {
        "errorReason.\(localizationKey)".localizedString
    }

    var localizationKey: String {
        switch self {
        case .setFirewallPolicy:
            "firewall"
        case .setRouting:
            "routing"
        case .setDns:
            "dns"
        case .inactiveSubscription:
            "subscriptionExpired"
        default:
            "\(self)"
        }
    }
}

private extension ErrorReason.Code {
    init(with errorStateReason: ErrorStateReason) {
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
        case .connectionAttemptsExceeded:
            self = .connectionAttemptsExceeded
        case .setLocalDnsResolverConfig:
            self = .setLocalDnsResolverConfig
        }
    }
}
