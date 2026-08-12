import NymVPNLib

public enum NymDeeplinkKind: Equatable, Hashable {
    case privy
    case privyLink
    case autologinRenew
    case autologinView
    case createAccount

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
        case .createAccount:
            .createAccount
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
        case .createAccount:
            self = .createAccount
        }
    }
}
