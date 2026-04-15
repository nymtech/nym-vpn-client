#if os(iOS)
import NymVPNLib
#elseif os(macOS)
import NymVPNRpc
#endif

#if os(iOS)
public typealias OuterSubscription = NymVPNLib.Subscription
#elseif os(macOS)
public typealias OuterSubscription = NymVPNRpc.Subscription
#endif

public struct Subscription {
    public let status: VpnSubscriptionStatus
    public let subscription: VpnSubscription

    public init(from subscription: OuterSubscription) {
        self.status = VpnSubscriptionStatus(from: subscription.status)
        self.subscription = VpnSubscription(from: subscription.subscription)
    }
}
