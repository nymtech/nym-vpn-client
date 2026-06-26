import Testing

struct XCStringsResolverTests {
    @Test func resolvesKnownKeys() throws {
        let resolver = try XCStringsResolver.default()
        #expect(resolver.string("planExpiresOn") == "Plan expires on")
        #expect(resolver.string("planValidUntil") == "Plan valid until")
        #expect(resolver.string("noActivePlan") == "This account has no active subscription")
        #expect(resolver.string("settings.logout") == "Log out")
    }

    @Test func resolvesOnboardingProcessingKeys() throws {
        let resolver = try XCStringsResolver.default()
        let keys = [
            "processingAccount.login.settingUp",
            "processingAccount.awaitingConfirmation.title",
            "processingAccount.awaitingConfirmation.subtitle"
        ]
        for key in keys {
            let value = resolver.string(key)
            #expect(value != key, "Catalog must define English for \(key)")
            #expect(!value.isEmpty)
        }
    }

    @Test func resolvesCheckoutAndIncompleteSubscriptionKeys() throws {
        let resolver = try XCStringsResolver.default()
        let keys = [
            "oneClick.incompleteSubscription.title",
            "oneClick.incompleteSubscription.message",
            "oneClick.incompleteSubscription.action",
            "purchasePlan.checkoutDismissed.title",
            "purchasePlan.checkoutDismissed.message",
            "subscriptionPurchase.choice.title",
            "subscriptionPurchase.choice.message",
            "subscriptionPurchase.choice.inApp",
            "subscriptionPurchase.choice.web"
        ]
        for key in keys {
            let value = resolver.string(key)
            #expect(value != key, "Catalog must define English for \(key)")
            #expect(!value.isEmpty)
        }
    }

    @Test func unknownKeyFallsBackToKey() throws {
        let resolver = try XCStringsResolver.default()
        #expect(resolver.string("totally.bogus.key") == "totally.bogus.key")
    }
}
