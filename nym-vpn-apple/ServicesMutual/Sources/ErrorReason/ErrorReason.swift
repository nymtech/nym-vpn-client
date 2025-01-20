import Foundation
#if os(iOS)
import MixnetLibrary
#endif
import Theme

public enum ErrorReason: LocalizedError {
    case firewall
    case routing
    case dns
    case tunDevice
    case tunnelProvider
    case internalUnknown
    case sameEntryAndExitGateway
    case invalidEntryGatewayCountry
    case invalidExitGatewayCountry
    case badBandwidthIncrease
    case duplicateTunFd
    case offline
    case unknown

    public static let domain = "ErrorHandler.ErrorReason"

#if os(iOS)
    public init(with errorStateReason: ErrorStateReason) {
        switch errorStateReason {
        case .firewall:
            self = .firewall
        case .routing:
            self = .routing
        case .dns:
            self = .dns
        case .tunDevice:
            self = .tunDevice
        case .tunnelProvider:
            self = .tunnelProvider
        case .internal:
            self = .internalUnknown
        case .sameEntryAndExitGateway:
            self = .sameEntryAndExitGateway
        case .invalidEntryGatewayCountry:
            self = .invalidEntryGatewayCountry
        case .invalidExitGatewayCountry:
            self = .invalidExitGatewayCountry
        case .badBandwidthIncrease:
            self = .badBandwidthIncrease
        case .duplicateTunFd:
            self = .duplicateTunFd
        }
    }
#endif

    public init?(nsError: NSError) {
        guard nsError.domain == ErrorReason.domain else { return nil }
        switch nsError.code {
        case 0:
            self = .firewall
        case 1:
            self = .routing
        case 2:
            self = .dns
        case 3:
            self = .tunDevice
        case 4:
            self = .tunnelProvider
        case 5:
            self = .internalUnknown
        case 6:
            self = .sameEntryAndExitGateway
        case 7:
            self = .invalidEntryGatewayCountry
        case 8:
            self = .invalidExitGatewayCountry
        case 9:
            self = .badBandwidthIncrease
        default:
            self = .unknown
        }
    }

    public var errorDescription: String? {
        description
    }

    public var nsError: NSError {
        let userInfo: [String: String] = [
            NSLocalizedDescriptionKey: description
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
        switch self {
        case .firewall:
            0
        case .routing:
            1
        case .dns:
            2
        case .tunDevice:
            3
        case .tunnelProvider:
            4
        case .internalUnknown:
            5
        case .sameEntryAndExitGateway:
            6
        case .invalidEntryGatewayCountry:
            7
        case .invalidExitGatewayCountry:
            8
        case .badBandwidthIncrease:
            9
        case .duplicateTunFd:
            10
        default:
            11
        }
    }
}

extension ErrorReason {
    private var description: String {
        switch self {
        case .firewall:
            "errorReason.firewall".localizedString
        case .routing:
            "errorReason.routing".localizedString
        case .dns:
            "errorReason.dns".localizedString
        case .tunDevice:
            "errorReason.tunDevice".localizedString
        case .tunnelProvider:
            "errorReason.tunnelProvider".localizedString
        case .internalUnknown:
            "errorReason.internalUnknown".localizedString
        case .sameEntryAndExitGateway:
            "errorReason.sameEntryAndExitGateway".localizedString
        case .invalidEntryGatewayCountry:
            "errorReason.invalidEntryGatewayCountry".localizedString
        case .invalidExitGatewayCountry:
            "errorReason.invalidExitGatewayCountry".localizedString
        case .badBandwidthIncrease:
            "errorReason.badBandwidthIncrease".localizedString
        case .duplicateTunFd:
            "errorReason.duplicateTunFd".localizedString
        case .offline:
            "errorReason.offline".localizedString
        case .unknown:
            "errorReason.unknown".localizedString
        }
    }
}
