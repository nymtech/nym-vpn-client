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
    case resolveGatewayAddrs
    case startLocalDnsResolver
    case unknown

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
        case .resolveGatewayAddrs:
            self = .resolveGatewayAddrs
        case .startLocalDnsResolver:
            self = .startLocalDnsResolver
        }
    }
#endif

    // swiftlint:disable:next function_body_length
    public init?(nsError: NSError) {
        guard nsError.domain == ErrorReason.domain,
            let errorReason = ErrorReason(nsError: nsError)
        else {
            self = .unknown
            return
        }

        switch ErrorReasonCode(errorReason: errorReason) {
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
        case .tunDevice:
            self = .tunDevice
        case .tunnelProvider:
            self = .tunnelProvider
        case .internalUnknown:
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
        case .syncAccount:
            self = .syncAccount(details: nsError.userInfo["details"] as? String ?? "Something went wrong.")
        case .syncDevice:
            self = .syncDevice(details: nsError.userInfo["details"] as? String ?? "Something went wrong.")
        case .registerDevice:
            self = .registerDevice(details: nsError.userInfo["details"] as? String ?? "Something went wrong.")
        case .requestZknym:
            self = .requestZknym(details: nsError.userInfo["details"] as? String ?? "Something went wrong.")
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
        case .resolveGatewayAddrs:
            self = .resolveGatewayAddrs
        case .startLocalDnsResolver:
            self = .startLocalDnsResolver
        case .none:
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
        ErrorReasonCode(errorReason: self)?.rawValue ?? 0
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
        case .resolveGatewayAddrs:
            "errorReason.resolveGatewayAddrs".localizedString
        case .startLocalDnsResolver:
            "errorReason.startLocalDnsResolver".localizedString
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
    case tunDevice
    case tunnelProvider
    case internalUnknown
    case sameEntryAndExitGateway
    case invalidEntryGatewayCountry
    case invalidExitGatewayCountry
    case badBandwidthIncrease
    case duplicateTunFd
    case syncAccount
    case syncDevice
    case registerDevice
    case requestZknym
    case requestZkNymBundle
    case resolveGatewayAddrs
    case startLocalDnsResolver

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
        case .tunDevice:
            self = .tunDevice
        case .tunnelProvider:
            self = .tunnelProvider
        case .internalUnknown:
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
        case .resolveGatewayAddrs:
            self = .resolveGatewayAddrs
        case .startLocalDnsResolver:
            self = .startLocalDnsResolver
        }
    }
}
