import Foundation

enum AppDrawerContent: Equatable {
    case welcome
    case processing
    case oneClick

    var isOneClick: Bool {
        if case .oneClick = self {
            return true
        }
        return false
    }

    var isWelcome: Bool {
        if case .welcome = self {
            return true
        }
        return false
    }

    var isProcessing: Bool {
        if case .processing = self {
            return true
        }
        return false
    }

    /// True when an imported credential should promote the drawer to
    /// `.processing` (and on completion to `.oneClick`). Pre-auth surfaces
    /// qualify; the post-auth `.processing` and `.oneClick` drawers do not.
    var allowsCredentialPromotion: Bool {
        switch self {
        case .welcome:
            return true
        case .processing, .oneClick:
            return false
        }
    }

    /// Coarse identity used by `DrawerView` to decide when to slide. Welcome
    /// and processing share the `preauth` identity so the drawer stays put
    /// and we can animate the swap internally instead of sliding the whole
    /// modal down/up.
    var slideID: AppDrawerSlideID {
        switch self {
        case .welcome, .processing:
            return .preauth
        case .oneClick:
            return .oneClick
        }
    }
}

enum AppDrawerSlideID: Hashable {
    case preauth
    case oneClick
}
