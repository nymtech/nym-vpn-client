import Foundation
import Testing
@testable import AccountPrefetchGates

struct EnvironmentChangeIAPPolicyTests {
    @Test func purchaseReadyTokenRequiresUUID() {
        #expect(!EnvironmentChangeIAPPolicy.hasPurchaseReadyToken(nil))
        #expect(!EnvironmentChangeIAPPolicy.hasPurchaseReadyToken(""))
        #expect(!EnvironmentChangeIAPPolicy.hasPurchaseReadyToken("not-a-uuid"))
        #expect(
            EnvironmentChangeIAPPolicy.hasPurchaseReadyToken(
                "740bf07e-9f8c-425b-9698-79c4a473f429"
            )
        )
    }

    @Test func reRegisterWhenCredentialImportedWithoutEnvToken() {
        #expect(
            EnvironmentChangeIAPPolicy.shouldReRegisterAccountAfterEnvironmentChange(
                isCredentialImported: true,
                tokenForTargetEnv: nil
            )
        )
        #expect(
            !EnvironmentChangeIAPPolicy.shouldReRegisterAccountAfterEnvironmentChange(
                isCredentialImported: false,
                tokenForTargetEnv: nil
            )
        )
        #expect(
            !EnvironmentChangeIAPPolicy.shouldReRegisterAccountAfterEnvironmentChange(
                isCredentialImported: true,
                tokenForTargetEnv: "740bf07e-9f8c-425b-9698-79c4a473f429"
            )
        )
    }
}

struct PostPurchaseProcessingPolicyTests {
    @Test func navigationRequiresSyncAndActiveAccount() {
        #expect(
            PostPurchaseProcessingPolicy.shouldCompleteNavigation(
                didSyncSubscription: true,
                isAccountActive: true
            )
        )
        #expect(
            !PostPurchaseProcessingPolicy.shouldCompleteNavigation(
                didSyncSubscription: true,
                isAccountActive: false
            )
        )
        #expect(
            !PostPurchaseProcessingPolicy.shouldCompleteNavigation(
                didSyncSubscription: false,
                isAccountActive: true
            )
        )
    }
}

struct PostPurchaseProcessingFlowTests {
    @Test func syncFailureSkipsPrefetch() async {
        let outcome = await AccountPrefetchOrchestrator.runPostPurchaseProcessingFlow(
            syncSubscriptionPayment: {
                throw URLError(.badServerResponse)
            },
            isAccountActive: { true },
            prefetchZkNyms: { .fetchedTickets }
        )
        #expect(!outcome.didSyncSummary)
        #expect(!outcome.isAccountActive)
        #expect(outcome.prefetchResult == nil)
    }

    @Test func syncSuccessPrefetchesWhenActive() async {
        let outcome = await AccountPrefetchOrchestrator.runPostPurchaseProcessingFlow(
            syncSubscriptionPayment: {},
            isAccountActive: { true },
            prefetchZkNyms: { .fetchedTickets }
        )
        #expect(outcome.didSyncSummary)
        #expect(outcome.isAccountActive)
        #expect(outcome.prefetchResult == .fetchedTickets)
    }

    @Test func syncSuccessLeavesInactiveWithoutPrefetch() async {
        let outcome = await AccountPrefetchOrchestrator.runPostPurchaseProcessingFlow(
            syncSubscriptionPayment: {},
            isAccountActive: { false },
            prefetchZkNyms: { .fetchedTickets }
        )
        #expect(outcome.didSyncSummary)
        #expect(!outcome.isAccountActive)
        #expect(outcome.prefetchResult == nil)
        #expect(
            !PostPurchaseProcessingPolicy.shouldCompleteNavigation(
                didSyncSubscription: outcome.didSyncSummary,
                isAccountActive: outcome.isAccountActive
            )
        )
    }
}
