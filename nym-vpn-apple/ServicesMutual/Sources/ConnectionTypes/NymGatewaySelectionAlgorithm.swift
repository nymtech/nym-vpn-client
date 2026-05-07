#if os(iOS)
import NymVPNLib
#elseif os(macOS)
import NymVPNRpc
#endif

public enum NymGatewaySelectionAlgorithm: String, Codable, Equatable, Sendable, CaseIterable {
    /// Select gateways explicitly using the entry and exit selectors.
    case explicit
    /// Select an exit explicitly while automatically selecting an entry gateway.
    case autoEntryExplicitExit
    /// Automatically select both entry and exit gateways. Forces 2-hop server-side.
    case auto
}

extension NymGatewaySelectionAlgorithm {
    public init(from sdk: GatewaySelectionAlgorithm) {
        switch sdk {
        case .explicit:
            self = .explicit
        case .autoEntryExplicitExit:
            self = .autoEntryExplicitExit
        case .auto:
            self = .auto
        }
    }

    public var sdkValue: GatewaySelectionAlgorithm {
        switch self {
        case .explicit:
            return .explicit
        case .autoEntryExplicitExit:
            return .autoEntryExplicitExit
        case .auto:
            return .auto
        }
    }
}
