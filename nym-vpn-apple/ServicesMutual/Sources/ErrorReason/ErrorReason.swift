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
    case syncAccount(details: String)
    case syncDevice(details: String)
    case registerDevice(details: String)
    case requestZknym(details: String)
    case requestZkNymBundle(successes: [String], failed: [String])
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
        case let .syncAccount(details: details):
            let messageString: String
            switch details {
            case .noAccountStored:
                messageString = "No account stored. Please add a mnemonic."
            case let .errorResponse(vpnApiErrorResponse):
                messageString = vpnApiErrorResponse.message
            case let .unexpectedResponse(message), let .internal(message):
                messageString = message
            }
            self = .syncAccount(details: messageString)
        case let .syncDevice(details: details):
            let messageString: String
            switch details {
            case .noAccountStored:
                messageString = "No account stored. Please add a mnemonic."
            case .noDeviceStored:
                messageString = "No device stored. Please reatry."
            case let .errorResponse(vpnApiErrorResponse):
                messageString = vpnApiErrorResponse.message
            case let .unexpectedResponse(message), let .internal(message):
                messageString = message
            }
            self = .syncDevice(details: messageString)
        case let .registerDevice(details: details):
            let messageString: String
            switch details {
            case .noAccountStored:
                messageString = "No account stored. Please add a mnemonic."
            case .noDeviceStored:
                messageString = "No device stored. Please reatry."
            case let .errorResponse(vpnApiErrorResponse):
                messageString = vpnApiErrorResponse.message
            case let .unexpectedResponse(message):
                messageString = message
            case let .internal(message):
                messageString = message
            }
            self = .registerDevice(details: messageString)
        case let .requestZkNym(details: details):
            let messageString: String
            switch details {
            case .noAccountStored:
                messageString = "No account stored. Please add a mnemonic."
            case .noDeviceStored:
                messageString = "No device stored. Please reatry."
            case let .vpnApi(vpnApiErrorResponse):
                messageString = vpnApiErrorResponse.message
            case let .unexpectedVpnApiResponse(message), let .storage(message), let .internal(message):
                messageString = message
            }
            self = .requestZknym(details: messageString)
        case let .requestZkNymBundle(successes: successes, failed: failed):
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
        case 10:
            self = .duplicateTunFd
        case 11:
            self = .syncAccount(details: "")
        case 12:
            self = .syncDevice(details: "")
        case 13:
            self = .registerDevice(details: "")
        case 14:
            self = .requestZknym(details: "")
        case 15:
            self = .requestZkNymBundle(successes: [], failed: [])
        case 16:
            self = .offline
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
        case .unknown:
            "errorReason.unknown".localizedString
        case let .syncAccount(details: details):
            details
        case let .syncDevice(details: details):
            details
        case let .registerDevice(details: details):
            details
        case let .requestZknym(details: details):
            details
        case let .requestZkNymBundle(successes: successes, failed: failed):
            "\(successes.first ?? "") \(failed.first ?? "")"
        case .offline:
            "errorReason.offline".localizedString
        }
    }
}
