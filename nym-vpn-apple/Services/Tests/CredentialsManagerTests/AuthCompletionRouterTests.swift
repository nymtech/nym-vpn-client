import Foundation
import Testing
import AccountPrefetchGates

struct AuthCompletionRouterTests {
    @Test func e3InactiveNewAccountRoutesToPurchaseWithoutProcessing() {
        #expect(
            AuthCompletionRouter.route(
                outcome: .registeredNeedsPurchase,
                flow: .createAccount
            ) == .routeToPurchase
        )
    }

    @Test func e4ActiveAccountAfterAuthStartsPostPurchaseProcessing() {
        #expect(
            AuthCompletionRouter.route(
                outcome: .registeredActive,
                flow: .createAccount
            ) == .startProcessing(.postPurchase)
        )
    }

    @Test func loginReadyUsesLoginProcessingCopy() {
        #expect(
            AuthCompletionRouter.route(
                outcome: .loginReady,
                flow: .login
            ) == .startProcessing(.login)
        )
    }

    @Test func loginReadyNeverRoutesToPurchase() {
        #expect(
            AuthCompletionRouter.route(
                outcome: .loginReady,
                flow: .login
            ) != .routeToPurchase
        )
    }
}

@MainActor
struct AuthCompletionOutcomeResolverTests {
    @Test func l1LoginInactiveBeforeSyncActiveAfterSyncYieldsLoginReady() async {
        var activeAfterSync = false
        var didSync = false
        let outcome = await AuthCompletionOutcomeResolver.resolve(
            flow: .login,
            isAccountActive: { activeAfterSync },
            updateAccountSummary: { untilActive in
                didSync = true
                #expect(untilActive)
                activeAfterSync = true
            }
        )
        #expect(didSync)
        #expect(outcome == .loginReady)
    }

    @Test func l2LoginInactiveAfterSyncYieldsNeedsPurchase() async {
        let outcome = await AuthCompletionOutcomeResolver.resolve(
            flow: .login,
            isAccountActive: { false },
            updateAccountSummary: { untilActive in
                #expect(untilActive)
            }
        )
        #expect(outcome == .registeredNeedsPurchase)
    }

    @Test func l3CreateAccountActiveAfterSyncYieldsRegisteredActive() async {
        let outcome = await AuthCompletionOutcomeResolver.resolve(
            flow: .createAccount,
            isAccountActive: { true },
            updateAccountSummary: { untilActive in
                #expect(!untilActive)
            }
        )
        #expect(outcome == .registeredActive)
    }
}

struct DrawerCredentialImportPolicyTests {
    @Test func e1AnonymousHandoffSuppressesDuplicateProcessingOnCredentialImport() {
        #expect(
            DrawerCredentialImportPolicy.action(
                imported: true,
                pendingAuthFlow: .createAccount,
                authHandoffCompleted: false,
                authHandoffCompletesOnCredentialImport: false,
                hasAccountToken: true,
                drawerAllowsCredentialPromotion: true
            ) == .suppressDuringHandoff
        )
    }

    @Test func privyCancelClearsHandoffSoImportDoesNotAutoComplete() {
        #expect(
            DrawerCredentialImportPolicy.action(
                imported: true,
                pendingAuthFlow: nil,
                authHandoffCompleted: false,
                authHandoffCompletesOnCredentialImport: false,
                hasAccountToken: true,
                drawerAllowsCredentialPromotion: true
            ) == .startExternalProcessing
        )
    }

    @Test func privySuccessCompletesOnCredentialImport() {
        #expect(
            DrawerCredentialImportPolicy.action(
                imported: true,
                pendingAuthFlow: .login,
                authHandoffCompleted: false,
                authHandoffCompletesOnCredentialImport: true,
                hasAccountToken: true,
                drawerAllowsCredentialPromotion: true
            ) == .completeAuthOnImport(.login)
        )
    }

    @Test func cancelledPrivyHandoffDoesNotCompleteOnImport() {
        #expect(
            DrawerCredentialImportPolicy.action(
                imported: true,
                pendingAuthFlow: nil,
                authHandoffCompleted: false,
                authHandoffCompletesOnCredentialImport: true,
                hasAccountToken: true,
                drawerAllowsCredentialPromotion: true
            ) == .startExternalProcessing
        )
    }

    @Test func externalImportWithoutHandoffStartsProcessingOnce() {
        #expect(
            DrawerCredentialImportPolicy.action(
                imported: true,
                pendingAuthFlow: nil,
                authHandoffCompleted: false,
                authHandoffCompletesOnCredentialImport: false,
                hasAccountToken: true,
                drawerAllowsCredentialPromotion: true
            ) == .startExternalProcessing
        )
    }

    @Test func failedPassphraseHandoffOrphanedSuppressesImportUntilCleared() {
        #expect(
            DrawerCredentialImportPolicy.action(
                imported: true,
                pendingAuthFlow: .login,
                authHandoffCompleted: false,
                authHandoffCompletesOnCredentialImport: false,
                hasAccountToken: true,
                drawerAllowsCredentialPromotion: true
            ) == .suppressDuringHandoff
        )
        #expect(
            DrawerCredentialImportPolicy.action(
                imported: true,
                pendingAuthFlow: nil,
                authHandoffCompleted: false,
                authHandoffCompletesOnCredentialImport: false,
                hasAccountToken: true,
                drawerAllowsCredentialPromotion: true
            ) == .startExternalProcessing
        )
    }

    @Test func completedHandoffDoesNotRestartProcessingOnImport() {
        #expect(
            DrawerCredentialImportPolicy.action(
                imported: true,
                pendingAuthFlow: nil,
                authHandoffCompleted: true,
                authHandoffCompletesOnCredentialImport: false,
                hasAccountToken: true,
                drawerAllowsCredentialPromotion: true
            ) == .none
        )
    }
}
