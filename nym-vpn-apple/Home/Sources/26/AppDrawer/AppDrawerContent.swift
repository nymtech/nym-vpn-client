import Foundation

enum AppDrawerContent: Equatable {
    case technicalOptIns
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
    /// `.processing` (and on completion to `.oneClick` or `.technicalOptIns`).
    /// Only `.welcome` qualifies; `.technicalOptIns` is reached after
    /// processing, and `.processing`/`.oneClick` are post-auth.
    var allowsCredentialPromotion: Bool {
        switch self {
        case .welcome:
            return true
        case .technicalOptIns, .processing, .oneClick:
            return false
        }
    }

    /// Coarse identity used by `DrawerView` to decide when to slide. Welcome,
    /// processing and opt-ins share the `preauth` identity so the drawer stays
    /// put across the auth → processing → tech-opt-ins handoff and we can
    /// animate swaps internally instead of sliding the modal down/up.
    var slideID: AppDrawerSlideID {
        switch self {
        case .welcome, .processing, .technicalOptIns:
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
