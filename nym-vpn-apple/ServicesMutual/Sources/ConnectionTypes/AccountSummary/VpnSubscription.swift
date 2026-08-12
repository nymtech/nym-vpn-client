import Foundation
import NymVPNLib

public struct VpnSubscription: Codable {
    public let createdOnUtc: Date?
    public let lastUpdatedUtc: Date?
    public let id: String
    public let validUntilDate: Date
    public let validFromDate: Date
    public let status: String
    public let kind: VpnSubscriptionKind
    public let isRecurring: Bool

    public init(
        createdOnUtc: Date?,
        lastUpdatedUtc: Date?,
        id: String,
        validUntilDate: Date,
        validFromDate: Date,
        status: String,
        kind: VpnSubscriptionKind,
        isRecurring: Bool
    ) {
        self.createdOnUtc = createdOnUtc
        self.lastUpdatedUtc = lastUpdatedUtc
        self.id = id
        self.validUntilDate = validUntilDate
        self.validFromDate = validFromDate
        self.status = status
        self.kind = kind
        self.isRecurring = isRecurring
    }

    public init(from subscription: NymVpnSubscription) {
        self.createdOnUtc = ISO8601DateFormatter().date(from: subscription.createdOnUtc)
        self.lastUpdatedUtc = ISO8601DateFormatter().date(from: subscription.lastUpdatedUtc)
        self.id = subscription.id
        self.validUntilDate = Date(timeIntervalSince1970: TimeInterval(subscription.validUntilUtc))
        self.validFromDate = Date(timeIntervalSince1970: TimeInterval(subscription.validFromUtc))
        self.status = subscription.status
        self.kind = VpnSubscriptionKind(from: subscription.kind)
        self.isRecurring = subscription.isRecurring
    }
}
