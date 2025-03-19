import Foundation
#if os(iOS)
import MixnetLibrary
#endif
import Theme

public enum ErrorReason: LocalizedError {
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
    case subscriptionExpired
    case api(String)
    case unknown

    private static let somethingWentWrong = "generalNymError.somethingWentWrong".localizedString

    public static let domain = "ErrorHandler.ErrorReason"

#if os(iOS)
    // swiftlint:disable:next function_body_length
    public init(with errorStateReason: ErrorStateReason) {
        switch errorStateReason {
        case .firewall:
            self = .firewall
        case .routing:
            self = .routing
        case .dns:
            self = .dns
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
        case .subscriptionExpired:
            self = .subscriptionExpired
        case let .api(message):
            self = .api(message ?? Self.somethingWentWrong)
        }
    }
#endif

    // swiftlint:disable:next function_body_length
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
        case .subscriptionExpired:
            self = .subscriptionExpired
        case .api:
            self = .api(nsError.userInfo["details"] as? String ?? Self.somethingWentWrong)
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
        case .subscriptionExpired:
            "errorReason.subscriptionExpired".localizedString
        case let .api(message):
            message
        }
    }
}

extension ErrorReason: Equatable {
    public static func == (lhs: ErrorReason, rhs: ErrorReason) -> Bool {
        lhs.errorCode == rhs.errorCode
    }
}

enum ErrorReasonCode: Int, RawRepresentable {
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
    case subscriptionExpired
    case api

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
        case .subscriptionExpired:
            self = .subscriptionExpired
        case .api:
            self = .api
        }
    }
}
