#if os(iOS)
import NymVPNLib
#elseif os(macOS)
import NymVPNRpc
#endif

public enum NymDeeplinkKind: Equatable, Hashable {
    case privy
    case privyLink
    case autologinRenew
    case autologinView

    public var deeplinkKind: DeeplinkKind {
        switch self {
        case .privy:
                .privy
        case .privyLink:
                .privyLink
        case .autologinRenew:
                .autologinRenew
        case .autologinView:
                .autologinView
        }
    }


    public init(from deeplinkKind: DeeplinkKind) {
        switch deeplinkKind {
        case .privy:
            self = .privy
        case .privyLink:
            self = .privyLink
        case .autologinRenew:
            self = .autologinRenew
        case .autologinView:
            self = .autologinView
        }
    }
}
