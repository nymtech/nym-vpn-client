import Foundation
import Testing
import AccountPrefetchGates

struct OnboardingAccountPreparationPolicyTests {
    @Test func inactiveSubscriptionIsPreparedForOnboarding() {
        let outcome = OnboardingAccountPreparationPolicy.waitOutcome(
            for: .error(.inactiveSubscription)
        )
        #expect(outcome == .prepared)
    }

    @Test func inactiveAccountStatusIsPreparedForOnboarding() {
        let outcome = OnboardingAccountPreparationPolicy.waitOutcome(
            for: .error(.accountStatusNotActive(status: "pending"))
        )
        #expect(outcome == .prepared)
    }

    @Test func readyToConnectIsPrepared() {
        #expect(
            OnboardingAccountPreparationPolicy.waitOutcome(for: .readyToConnect) == .prepared
        )
    }

    @Test func syncingContinuesWaiting() {
        #expect(
            OnboardingAccountPreparationPolicy.waitOutcome(for: .syncing) == .continueWaiting
        )
    }

    @Test func storageErrorFailsWithStructuredMessage() {
        let outcome = OnboardingAccountPreparationPolicy.waitOutcome(
            for: .error(.storage(context: "disk", details: "full"))
        )
        #expect(
            outcome == .fail("Storage error: disk - full ")
        )
    }

    @Test func apiFailureUsesStructuredMessage() {
        let message = OnboardingAccountPreparationPolicy.userFacingMessage(
            for: .apiFailure(context: "register_device", details: "timeout")
        )
        #expect(message == "API failure: register_device - timeout")
    }

    @Test func maxDeviceReachedFailsWithKnownCopy() {
        let outcome = OnboardingAccountPreparationPolicy.waitOutcome(
            for: .error(.maxDeviceReached)
        )
        #expect(outcome == .fail("Max device numbers reached"))
    }

    @Test func offlineDebounce_transientOfflineDoesNotFailImmediately() {
        #expect(
            !OnboardingAccountPreparationPolicy.shouldFailOnOffline(consecutiveOfflineSeconds: 3)
        )
        #expect(
            !OnboardingAccountPreparationPolicy.shouldFailOnOffline(
                consecutiveOfflineSeconds: OnboardingAccountPreparationPolicy.offlineFailDebounceSeconds - 0.25
            )
        )
    }

    @Test func offlineDebounce_sustainedOfflineFails() {
        #expect(
            OnboardingAccountPreparationPolicy.shouldFailOnOffline(
                consecutiveOfflineSeconds: OnboardingAccountPreparationPolicy.offlineFailDebounceSeconds
            )
        )
        #expect(
            OnboardingAccountPreparationPolicy.shouldFailOnOffline(consecutiveOfflineSeconds: 6)
        )
    }

    @Test func offlineDebounce_resetsAfterNonOfflinePoll() {
        let interval = OnboardingAccountPreparationPolicy.waitPollIntervalSeconds
        var streak: TimeInterval = 4.75
        streak += interval
        #expect(OnboardingAccountPreparationPolicy.shouldFailOnOffline(consecutiveOfflineSeconds: streak))
        streak = 0
        streak += interval
        #expect(!OnboardingAccountPreparationPolicy.shouldFailOnOffline(consecutiveOfflineSeconds: streak))
    }
}
