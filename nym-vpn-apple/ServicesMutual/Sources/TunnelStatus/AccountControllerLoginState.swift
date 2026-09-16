import Foundation

/// Shared login-inactive discriminant. Platforms map UniFFI/phase into this; they do not
/// each decide which controller errors finish login.
public enum AccountControllerLoginState: Equatable, Sendable {
    case inactiveSubscription
    case accountStatusNotActive
    case other

    public var isTerminalInactiveForLogin: Bool {
        switch self {
        case .inactiveSubscription, .accountStatusNotActive:
            true
        case .other:
            false
        }
    }
}
