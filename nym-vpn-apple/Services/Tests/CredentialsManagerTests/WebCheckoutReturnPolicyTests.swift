import Foundation
import Testing
import AccountPrefetchGates

struct WebCheckoutReturnPolicyTests {
    @Test func dismissesOnSubscriptionPaymentReturn() {
        // swiftlint:disable:next force_unwrapping
        let url = URL(string: "nymvpn://account/response")!
        #expect(WebCheckoutReturnPolicy.shouldDismissOnDeeplink(url: url))
    }

    @Test func doesNotDismissOnAccountImportReturn() {
        // swiftlint:disable:next force_unwrapping
        let url = URL(string: "nymvpn://account/response?deeplink_id=abc&payload=xyz")!
        #expect(!WebCheckoutReturnPolicy.shouldDismissOnDeeplink(url: url))
    }

    @Test func doesNotDismissOnUnrelatedDeeplink() {
        // swiftlint:disable:next force_unwrapping
        let url = URL(string: "nymvpn://auth/privy/privateKey")!
        #expect(!WebCheckoutReturnPolicy.shouldDismissOnDeeplink(url: url))
    }
}
