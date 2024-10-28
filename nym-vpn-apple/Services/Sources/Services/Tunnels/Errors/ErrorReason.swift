#if os(iOS)
import Foundation
import MixnetLibrary

public enum ErrorReason: String, Codable, Error {
    case firewall
    case routing
    case dns
    case tunDevice
    case tunnelProvider
    case establishMixnetConnection
    case establishWireguardConnection
    case tunnelDown
    case internalUnknown

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
        case .establishMixnetConnection:
            self = .establishMixnetConnection
        case .establishWireguardConnection:
            self = .establishWireguardConnection
        case .tunnelDown:
            self = .tunnelDown
        case .internal:
            self = .internalUnknown
        }
    }

    public init(from data: Data) throws {
        self = try JSONDecoder().decode(ErrorReason.self, from: data)
    }

    public func encode() throws -> Data {
        try JSONEncoder().encode(self)
    }
}

extension ErrorReason: LocalizedError {
    public var errorDescription: String? {
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
        case .establishMixnetConnection:
            "errorReason.establishMixnetConnection".localizedString
        case .establishWireguardConnection:
            "errorReason.establishWireguardConnection".localizedString
        case .tunnelDown:
            "errorReason.tunnelDown".localizedString
        case .internalUnknown:
            "errorReason.internalUnknown".localizedString
        }
    }
}
#endif
