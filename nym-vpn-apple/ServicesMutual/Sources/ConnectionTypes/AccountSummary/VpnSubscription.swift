import Foundation
#if os(iOS)
import NymVPNLib
#elseif os(macOS)
import NymVPNRpc
#endif

public struct VpnSubscription {
    public let createdOnUtc: Date?
    public let lastUpdatedUtc: Date?
    public let id: String
    public let validUntilDate: Date
    public let validFromDate: Date
    public let status: String
    public let kind: VpnSubscriptionKind
    public let isRecurring: Bool
    
    init(from subscription: NymVpnSubscription) {
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
