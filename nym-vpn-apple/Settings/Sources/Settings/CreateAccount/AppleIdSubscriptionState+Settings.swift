#if os(iOS)
import Foundation
import CredentialsManager
import PurchasesManager
import Theme

extension PurchasesManager {
    func appleIdSubscriptionState(for credentialsManager: CredentialsManager) -> AppleIdSubscriptionState {
        AppleIdSubscriptionPolicy.state(
            entitlement: appleIdEntitlement,
            accountToken: credentialsManager.accountToken,
            isAccountActive: credentialsManager.isAccountActive()
        )
    }
}

extension AppleIdSubscriptionState {
    /// Replaces the "no active subscription" copy; nil keeps today's copy.
    var localizedExplanation: String? {
        switch self {
        case .unchanged:
            nil
        case .confirmingPayment:
            "confirmingPayment".localizedString
        case let .onAnotherAccount(activeUntil, switchesToThisAccountOn):
            if let switchesToThisAccountOn {
                String(
                    format: "appleIdSubscription.switchesToThisAccount".localizedString,
                    switchesToThisAccountOn.formatted(date: .long, time: .omitted)
                )
            } else {
                String(
                    format: "appleIdSubscription.onAnotherAccount".localizedString,
                    activeUntil.formatted(date: .long, time: .omitted)
                )
            }
        }
    }

    /// Buying from this state can only be another uncharged plan change against the other account.
    var blocksPlanPurchase: Bool {
        if case .onAnotherAccount = self { return true }
        return false
    }
}
#endif
