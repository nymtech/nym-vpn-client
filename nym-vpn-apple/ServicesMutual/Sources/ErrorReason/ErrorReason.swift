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
                self = .noAccountStored
                return
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
                self = .noAccountStored
                return
            case .noDeviceStored:
                self = .noDeviceStored
                return
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
                self = .noAccountStored
                return
            case .noDeviceStored:
                self = .noDeviceStored
                return
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
                self = .noAccountStored
                return
            case .noDeviceStored:
                self = .noDeviceStored
                return
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
            self = .syncAccount(details: nsError.userInfo["details"] as? String ?? "Something went wrong.")
        case 12:
            self = .syncDevice(details: nsError.userInfo["details"] as? String ?? "Something went wrong.")
        case 13:
            self = .registerDevice(details: nsError.userInfo["details"] as? String ?? "Something went wrong.")
        case 14:
            self = .requestZknym(details: nsError.userInfo["details"] as? String ?? "Something went wrong.")
        case 15:
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
        case 16:
            self = .unknown
        case 17:
            self = .offline
        case 18:
            self = .noAccountStored
        case 19:
            self = .noDeviceStored
        default:
            self = .unknown
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
        case .syncAccount:
            11
        case .syncDevice:
            12
        case .registerDevice:
            13
        case .requestZknym:
            14
        case .requestZkNymBundle:
            15
        case .unknown:
            16
        case .offline:
            17
        case .noAccountStored:
            18
        case .noDeviceStored:
            19
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

private extension ErrorReason {
    var description: String {
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
        case .noAccountStored:
            "errorReason.noAccountStored".localizedString
        case .noDeviceStored:
            "errorReason.noDeviceStored".localizedString
        }
    }
}

extension ErrorReason: Equatable {
    public static func == (lhs: ErrorReason, rhs: ErrorReason) -> Bool {
        lhs.errorCode == rhs.errorCode
    }
}
