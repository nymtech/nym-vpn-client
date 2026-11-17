import Foundation

public enum FreepassError: Equatable, Sendable {
    case invalidCode
    case alreadyRedeemed
    case subscriptionActive
    case accountNotRegistered
    case rateLimited
    /// The tunnel holds the account store while connected.
    case vpnConnected
    case offline
    case server
    case unknown

    // messageIds from websites www/vpn-api freepass/redeem and account/[accountId]/freepass routes.
    static let apiMessages: [(text: String, error: FreepassError)] = [
        ("redeem-freepass.invalid", .invalidCode),
        ("apply-freepass.invalid-code", .invalidCode),
        ("redeem-freepass.already-redeemed", .alreadyRedeemed),
        ("apply-freepass.already-redeemed", .alreadyRedeemed),
        ("create-subscription.subscription-already-exists", .subscriptionActive),
        ("rate_limit_exceeded", .rateLimited),
        // The auth middleware sends no messageId for an account the API hasn't seen yet.
        ("Account not found", .accountNotRegistered)
    ]

    static func fromApi(messageId: String?, message: String) -> FreepassError {
        let texts = [messageId, message].compactMap { $0 }
        return apiMessages.first { entry in texts.contains { $0.contains(entry.text) } }?.error ?? .server
    }

    public var isFinal: Bool {
        self == .invalidCode || self == .alreadyRedeemed || self == .subscriptionActive
    }

    var isRejection: Bool {
        self != .server && self != .unknown
    }
}
