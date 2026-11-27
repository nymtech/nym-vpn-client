import Foundation

public enum FreepassError: Equatable, Sendable {
    case invalidCode
    case alreadyRedeemed
    case subscriptionActive
    case rateLimited
    /// The tunnel holds the account store while connected.
    case vpnConnected
    case offline
    case server
    case unknown

    public var isFinal: Bool {
        self == .invalidCode || self == .alreadyRedeemed || self == .subscriptionActive
    }
}
