import Foundation
import Testing
import AccountPrefetchGates

struct OnboardingSessionPolicyTests {
    @Test func progressStepMapping() {
        #expect(OnboardingSessionPolicy.progressStep(for: .creatingMnemonic) == 1)
        #expect(OnboardingSessionPolicy.progressStep(for: .registeringAccount) == 2)
        #expect(OnboardingSessionPolicy.progressStep(for: .iapPurchaseRequired) == 3)
        #expect(OnboardingSessionPolicy.progressStep(for: .processingPayment) == 4)
        #expect(OnboardingSessionPolicy.progressStep(for: .prefetchingZkNyms) == 4)
        #expect(OnboardingSessionPolicy.progressStep(for: .ready) == 4)
    }

    @Test func readyPhaseDoesNotRegress() {
        for phase in OnboardingPhase.allCases where phase != .ready {
            #expect(!OnboardingSessionPolicy.canTransition(from: .ready, to: phase))
        }
        #expect(OnboardingSessionPolicy.canTransition(from: .ready, to: .ready) == false)
    }

    @Test func phasesAdvanceForwardOnly() {
        #expect(OnboardingSessionPolicy.canTransition(from: .creatingMnemonic, to: .registeringAccount))
        #expect(OnboardingSessionPolicy.canTransition(from: .registeringAccount, to: .iapPurchaseRequired))
        #expect(!OnboardingSessionPolicy.canTransition(from: .iapPurchaseRequired, to: .creatingMnemonic))
        #expect(!OnboardingSessionPolicy.canTransition(from: .processingPayment, to: .registeringAccount))
    }

    @Test func purchaseOutcomeSkipsDrawerProcessing() {
        #expect(
            OnboardingSessionPolicy.processingFlow(
                for: .registeredNeedsPurchase,
                authFlow: .createAccount
            ) == .none
        )
        #expect(DrawerSessionPolicy.shouldRouteToPurchase(outcome: .registeredNeedsPurchase))
        #expect(!DrawerSessionPolicy.shouldStartDrawerProcessing(outcome: .registeredNeedsPurchase))
    }

    @Test func activeOutcomeUsesPostPurchaseProcessing() {
        #expect(
            OnboardingSessionPolicy.processingFlow(
                for: .registeredActive,
                authFlow: .createAccount
            ) == .postPurchase
        )
        #expect(DrawerSessionPolicy.shouldStartDrawerProcessing(outcome: .registeredActive))
    }

    @Test func loginOutcomeUsesLoginProcessing() {
        #expect(
            OnboardingSessionPolicy.processingFlow(
                for: .loginReady,
                authFlow: .login
            ) == .login
        )
    }
}

struct DrawerSessionPolicyTests {
    @Test func credentialImportAloneDoesNotStartProcessingDuringHandoff() {
        #expect(
            !DrawerSessionPolicy.shouldStartProcessingOnCredentialImport(
                isCredentialImported: true,
                hasAccountToken: true,
                authHandoffInProgress: true,
                authHandoffCompleted: false,
                drawerAllowsCredentialPromotion: true
            )
        )
        #expect(
            !DrawerSessionPolicy.shouldStartProcessingOnCredentialImport(
                isCredentialImported: true,
                hasAccountToken: true,
                authHandoffInProgress: false,
                authHandoffCompleted: true,
                drawerAllowsCredentialPromotion: true
            )
        )
    }

    @Test func externalCredentialImportCanStartProcessing() {
        #expect(
            DrawerSessionPolicy.shouldStartProcessingOnCredentialImport(
                isCredentialImported: true,
                hasAccountToken: true,
                authHandoffInProgress: false,
                authHandoffCompleted: false,
                drawerAllowsCredentialPromotion: true
            )
        )
    }

    @Test func credentialImportWithoutTokenDoesNotStartProcessing() {
        #expect(
            !DrawerSessionPolicy.shouldStartProcessingOnCredentialImport(
                isCredentialImported: true,
                hasAccountToken: false,
                authHandoffInProgress: false,
                authHandoffCompleted: false,
                drawerAllowsCredentialPromotion: true
            )
        )
    }

    @Test func importedCredentialDuringExplicitHandoffDoesNotStartProcessing() {
        #expect(
            !DrawerSessionPolicy.shouldStartProcessingOnCredentialImport(
                isCredentialImported: true,
                hasAccountToken: true,
                authHandoffInProgress: true,
                authHandoffCompleted: false,
                drawerAllowsCredentialPromotion: true
            )
        )
    }
}
