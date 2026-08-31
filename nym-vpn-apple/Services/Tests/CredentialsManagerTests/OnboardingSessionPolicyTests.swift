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

    @Test func loginReadyNeverOffersPurchaseAfterAuth() {
        #expect(!DrawerSessionPolicy.shouldOfferPlanPurchaseAfterAuth(outcome: .loginReady))
        #expect(!DrawerSessionPolicy.shouldOfferPlanPurchaseAfterAuth(outcome: .registeredActive))
        #expect(DrawerSessionPolicy.shouldOfferPlanPurchaseAfterAuth(outcome: .registeredNeedsPurchase))
        #expect(!DrawerSessionPolicy.shouldOfferPlanPurchaseAfterAuth(outcome: nil))
    }

    @Test func loginProcessingOffersPurchaseOnlyWhenStillInactiveAfterSync() {
        #expect(
            !DrawerSessionPolicy.shouldOfferPlanPurchaseAfterProcessing(
                processingKind: .login,
                authOutcome: .loginReady,
                isAccountActive: true
            )
        )
        #expect(
            DrawerSessionPolicy.shouldOfferPlanPurchaseAfterProcessing(
                processingKind: .login,
                authOutcome: .loginReady,
                isAccountActive: false
            )
        )
        #expect(
            !DrawerSessionPolicy.shouldOfferPlanPurchaseAfterProcessing(
                processingKind: .login,
                authOutcome: .registeredNeedsPurchase,
                isAccountActive: true
            )
        )
        #expect(
            DrawerSessionPolicy.shouldOfferPlanPurchaseAfterProcessing(
                processingKind: .login,
                authOutcome: .registeredNeedsPurchase,
                isAccountActive: false,
                validUntilIsFuture: false,
                hasAccountSummary: false
            )
        )
        #expect(
            DrawerSessionPolicy.shouldOfferPlanPurchaseAfterProcessing(
                processingKind: .postPurchase,
                authOutcome: .registeredNeedsPurchase,
                isAccountActive: false
            )
        )
        #expect(
            !DrawerSessionPolicy.shouldOfferPlanPurchaseAfterProcessing(
                processingKind: .postPurchase,
                authOutcome: .registeredActive,
                isAccountActive: false
            )
        )
        #expect(
            !DrawerSessionPolicy.shouldOfferPlanPurchaseAfterProcessing(
                processingKind: .postPurchase,
                authOutcome: .registeredNeedsPurchase,
                isAccountActive: true
            )
        )
    }

    @Test func loginProcessingOffersPurchaseWhenSummaryMissing() {
        #expect(
            DrawerSessionPolicy.shouldOfferPlanPurchaseAfterProcessing(
                processingKind: .login,
                authOutcome: .registeredNeedsPurchase,
                isAccountActive: false,
                accountSummaryLastFetchFailed: false,
                validUntilIsFuture: false,
                hasAccountSummary: false
            )
        )
    }

    @Test func loginProcessingSkipsPurchaseWhenSummaryFetchFailed() {
        #expect(
            !DrawerSessionPolicy.shouldOfferPlanPurchaseAfterProcessing(
                processingKind: .login,
                authOutcome: .registeredNeedsPurchase,
                isAccountActive: false,
                accountSummaryLastFetchFailed: true,
                validUntilIsFuture: false,
                hasAccountSummary: false
            )
        )
    }

    @Test func loginProcessingSkipsPurchaseWhenValidUntilIsFuture() {
        #expect(
            !DrawerSessionPolicy.shouldOfferPlanPurchaseAfterProcessing(
                processingKind: .login,
                authOutcome: .registeredNeedsPurchase,
                isAccountActive: false,
                accountSummaryLastFetchFailed: false,
                validUntilIsFuture: true,
                hasAccountSummary: true
            )
        )
    }

    @Test func loginProcessingOffersPurchaseWhenGenuinelyInactive() {
        #expect(
            DrawerSessionPolicy.shouldOfferPlanPurchaseAfterProcessing(
                processingKind: .login,
                authOutcome: .registeredNeedsPurchase,
                isAccountActive: false,
                accountSummaryLastFetchFailed: false,
                validUntilIsFuture: false,
                hasAccountSummary: true
            )
        )
    }
}

struct LoginSessionPolicyTests {
    @Test func validUntilFutureIsEffectivelyActive() {
        let future = Date().addingTimeInterval(86_400)
        #expect(LoginSessionPolicy.validUntilIsFuture(validUntil: future))
        #expect(
            LoginSessionPolicy.isEffectivelyActive(
                isAccountActive: false,
                validUntilIsFuture: true,
                hasAccountSummary: true
            )
        )
    }
}

struct ConnectPlanPurchaseGatePolicyTests {
    @Test func connectSkipsPurchaseDuringRegistration() {
        #expect(
            !ConnectPlanPurchaseGatePolicy.shouldOfferPlanPurchaseOnConnect(
                isAccountRegistrationInFlight: true,
                accountSummaryLastFetchFailed: false,
                isAccountActive: false,
                validUntilIsFuture: false,
                hasAccountSummary: false
            )
        )
    }

    @Test func connectSkipsPurchaseWhenFetchFailed() {
        #expect(
            !ConnectPlanPurchaseGatePolicy.shouldOfferPlanPurchaseOnConnect(
                isAccountRegistrationInFlight: false,
                accountSummaryLastFetchFailed: true,
                isAccountActive: false,
                validUntilIsFuture: false,
                hasAccountSummary: false
            )
        )
    }

    @Test func connectSkipsPurchaseWhenValidUntilIsFuture() {
        #expect(
            !ConnectPlanPurchaseGatePolicy.shouldOfferPlanPurchaseOnConnect(
                isAccountRegistrationInFlight: false,
                accountSummaryLastFetchFailed: false,
                isAccountActive: false,
                validUntilIsFuture: true,
                hasAccountSummary: true
            )
        )
    }
}

struct ProcessingUIPolicyTests {
    @Test func staticProcessingHidesProgressBar() {
        #expect(!ProcessingUIPolicy.showsOnboardingProgressBar(usesStaticCopy: true))
        #expect(ProcessingUIPolicy.showsOnboardingProgressBar(usesStaticCopy: false))
    }
}

struct PurchasePresentationPolicyTests {
    @Test func progressBarVisibleDuringCarousel() {
        #expect(
            PurchasePresentationPolicy.showsOnboardingProgressBar(
                isPurchaseOnly: false,
                didFinishAnimatingText: false,
                didRegisterAccount: false
            )
        )
        #expect(
            PurchasePresentationPolicy.showsOnboardingProgressBar(
                isPurchaseOnly: false,
                didFinishAnimatingText: true,
                didRegisterAccount: false
            )
        )
    }

    @Test func progressBarHiddenOnPurchasePanel() {
        #expect(
            !PurchasePresentationPolicy.showsOnboardingProgressBar(
                isPurchaseOnly: true,
                didFinishAnimatingText: true,
                didRegisterAccount: true
            )
        )
        #expect(
            !PurchasePresentationPolicy.showsOnboardingProgressBar(
                isPurchaseOnly: false,
                didFinishAnimatingText: true,
                didRegisterAccount: true
            )
        )
    }

    @Test func purchasePanelVisibility() {
        #expect(
            PurchasePresentationPolicy.showsPurchasePanel(
                isPurchaseOnly: true,
                didFinishAnimatingText: false,
                didRegisterAccount: false
            )
        )
        #expect(
            PurchasePresentationPolicy.showsPurchasePanel(
                isPurchaseOnly: false,
                didFinishAnimatingText: true,
                didRegisterAccount: true
            )
        )
        #expect(
            !PurchasePresentationPolicy.showsPurchasePanel(
                isPurchaseOnly: false,
                didFinishAnimatingText: false,
                didRegisterAccount: false
            )
        )
    }
}

struct IAPFeedbackPolicyTests {
    @Test func incompleteSubscriptionBannerWhenImportedAndInactive() {
        #expect(
            IAPFeedbackPolicy.shouldShowIncompleteSubscriptionBanner(
                isCredentialImported: true,
                isAccountActive: false
            )
        )
        #expect(
            !IAPFeedbackPolicy.shouldShowIncompleteSubscriptionBanner(
                isCredentialImported: true,
                isAccountActive: true
            )
        )
        #expect(
            !IAPFeedbackPolicy.shouldShowIncompleteSubscriptionBanner(
                isCredentialImported: false,
                isAccountActive: false
            )
        )
    }

    @Test func checkoutDismissedFeedbackWhenImportedAndInactive() {
        #expect(
            IAPFeedbackPolicy.shouldShowCheckoutDismissedFeedback(
                isCredentialImported: true,
                isAccountActive: false
            )
        )
    }

    @Test func checkoutAlertsForUnhappyPathsOnly() {
        #expect(!IAPFeedbackPolicy.requiresUserAlert(for: .success))
        #expect(IAPFeedbackPolicy.requiresUserAlert(for: .userCancelled))
        #expect(IAPFeedbackPolicy.requiresUserAlert(for: .pending))
        #expect(IAPFeedbackPolicy.requiresUserAlert(for: .failed))
    }

    @Test func checkoutAlertLocalizationKeys() {
        #expect(
            IAPFeedbackPolicy.alertLocalizationKey(for: .userCancelled)
                == "purchasePlan.paymentCancelledAlert"
        )
        #expect(
            IAPFeedbackPolicy.alertLocalizationKey(for: .pending)
                == "purchasePlan.paymentPendingAlert"
        )
        #expect(
            IAPFeedbackPolicy.alertLocalizationKey(for: .failed)
                == "purchasePlan.paymentFailedAlert"
        )
    }

    @Test func purchaseCheckoutThrownErrorsUseFailedAlertKey() {
        #expect(
            IAPFeedbackPolicy.alertLocalizationKey(for: .failed)
                == "purchasePlan.paymentFailedAlert"
        )
    }

    @Test func registrationRetryOnlyOfferedForRegistrationFailures() {
        #expect(IAPFeedbackPolicy.showsRegistrationRetryOnAlert(isRegistrationFailure: true))
        #expect(!IAPFeedbackPolicy.showsRegistrationRetryOnAlert(isRegistrationFailure: false))
    }
}

struct DrawerSessionPolicyTests {
    @Test func planPurchaseTransitionIgnoredWhileAlreadyActive() {
        #expect(!DrawerSessionPolicy.shouldBeginPlanPurchaseTransition(isPurchaseFlowActive: true))
        #expect(DrawerSessionPolicy.shouldBeginPlanPurchaseTransition(isPurchaseFlowActive: false))
    }

    @Test func purchaseTransitionOverlayOnlyWhileDrawerHidden() {
        #expect(
            DrawerSessionPolicy.showsPurchaseTransitionOverlay(
                isPurchaseFlowActive: true,
                isDrawerContentNil: true
            )
        )
        #expect(
            !DrawerSessionPolicy.showsPurchaseTransitionOverlay(
                isPurchaseFlowActive: true,
                isDrawerContentNil: false
            )
        )
        #expect(
            !DrawerSessionPolicy.showsPurchaseTransitionOverlay(
                isPurchaseFlowActive: false,
                isDrawerContentNil: true
            )
        )
        #expect(
            !DrawerSessionPolicy.showsPurchaseTransitionOverlay(
                isPurchaseFlowActive: false,
                isDrawerContentNil: false
            )
        )
        #expect(
            DrawerSessionPolicy.showsPurchaseTransitionOverlay(
                isPurchaseFlowActive: true,
                isDrawerContentNil: false,
                isCheckoutNavigationPending: true
            )
        )
    }

    @Test func foregroundRefreshBypassesThrottleDuringPurchaseOrInactiveAccount() {
        #expect(
            DrawerSessionPolicy.shouldBypassForegroundAccountRefreshThrottle(
                isPurchaseFlowActive: true,
                isAccountActive: true
            )
        )
        #expect(
            DrawerSessionPolicy.shouldBypassForegroundAccountRefreshThrottle(
                isPurchaseFlowActive: false,
                isAccountActive: false
            )
        )
        #expect(
            !DrawerSessionPolicy.shouldBypassForegroundAccountRefreshThrottle(
                isPurchaseFlowActive: false,
                isAccountActive: true
            )
        )
    }

    @Test func checkoutCompletesAfterAccountRefreshWhenPurchaseActiveAndAccountActive() {
        #expect(
            DrawerSessionPolicy.shouldCompleteCheckoutAfterAccountRefresh(
                isPurchaseFlowActive: true,
                isAccountActive: true
            )
        )
        #expect(
            !DrawerSessionPolicy.shouldCompleteCheckoutAfterAccountRefresh(
                isPurchaseFlowActive: true,
                isAccountActive: false
            )
        )
        #expect(
            !DrawerSessionPolicy.shouldCompleteCheckoutAfterAccountRefresh(
                isPurchaseFlowActive: false,
                isAccountActive: true
            )
        )
    }

    @Test func purchaseDismissKeepsImportedAccountOnDashboard() {
        #expect(
            DrawerSessionPolicy.drawerDestinationAfterPurchaseDismiss(
                isCredentialImported: true,
                welcomeScreenDidDisplay: true
            ) == .oneClick
        )
        #expect(
            DrawerSessionPolicy.drawerDestinationAfterPurchaseDismiss(
                isCredentialImported: true,
                welcomeScreenDidDisplay: false
            ) == .technicalOptIns
        )
    }

    @Test func purchaseDismissWithoutCredentialReturnsWelcome() {
        #expect(
            DrawerSessionPolicy.drawerDestinationAfterPurchaseDismiss(
                isCredentialImported: false,
                welcomeScreenDidDisplay: true
            ) == .welcome
        )
    }

    @Test func incompleteImportWithCredentialNeverReturnsWelcome() {
        #expect(
            DrawerSessionPolicy.drawerDestinationAfterIncompleteCredentialImport(
                isCredentialImported: true,
                welcomeScreenDidDisplay: true
            ) == .oneClick
        )
        #expect(
            DrawerSessionPolicy.drawerDestinationAfterIncompleteCredentialImport(
                isCredentialImported: true,
                welcomeScreenDidDisplay: false
            ) == .technicalOptIns
        )
        #expect(
            DrawerSessionPolicy.drawerDestinationAfterIncompleteCredentialImport(
                isCredentialImported: false,
                welcomeScreenDidDisplay: false
            ) == nil
        )
    }

    @Test func importFailureDoesNotRegressWelcomeDuringActiveHandoff() {
        #expect(
            !DrawerSessionPolicy.shouldRegressToWelcomeAfterImportFailure(
                isCredentialImported: false,
                authHandoffInProgress: true
            )
        )
        #expect(
            DrawerSessionPolicy.shouldRegressToWelcomeAfterImportFailure(
                isCredentialImported: false,
                authHandoffInProgress: false
            )
        )
        #expect(
            !DrawerSessionPolicy.shouldRegressToWelcomeAfterImportFailure(
                isCredentialImported: true,
                authHandoffInProgress: false
            )
        )
    }

    @Test func authWillBeginPromotesPreImportedPrivyHandoff() {
        #expect(
            DrawerSessionPolicy.shouldBeginCredentialImportCompletionOnAuthWillBegin(
                completesOnCredentialImport: true,
                isCredentialImported: true,
                pendingAuthFlow: .login,
                authHandoffCompleted: false
            )
        )
        #expect(
            !DrawerSessionPolicy.shouldBeginCredentialImportCompletionOnAuthWillBegin(
                completesOnCredentialImport: true,
                isCredentialImported: true,
                pendingAuthFlow: .login,
                authHandoffCompleted: true
            )
        )
        #expect(
            !DrawerSessionPolicy.shouldBeginCredentialImportCompletionOnAuthWillBegin(
                completesOnCredentialImport: true,
                isCredentialImported: false,
                pendingAuthFlow: .login,
                authHandoffCompleted: false
            )
        )
        #expect(
            !DrawerSessionPolicy.shouldBeginCredentialImportCompletionOnAuthWillBegin(
                completesOnCredentialImport: false,
                isCredentialImported: true,
                pendingAuthFlow: .login,
                authHandoffCompleted: false
            )
        )
    }

    @Test func privyCreateAccountImportRequiresRegistrationWhenTokenMissing() {
        #expect(
            DrawerSessionPolicy.shouldRegisterAccountAfterCredentialImport(
                flow: .createAccount,
                accountToken: nil
            )
        )
        #expect(
            DrawerSessionPolicy.shouldRegisterAccountAfterCredentialImport(
                flow: .createAccount,
                accountToken: ""
            )
        )
        #expect(
            !DrawerSessionPolicy.shouldRegisterAccountAfterCredentialImport(
                flow: .createAccount,
                accountToken: "token"
            )
        )
        #expect(
            DrawerSessionPolicy.shouldRegisterAccountAfterCredentialImport(
                flow: .login,
                accountToken: nil
            )
        )
        #expect(
            !DrawerSessionPolicy.shouldRegisterAccountAfterCredentialImport(
                flow: .login,
                accountToken: "token"
            )
        )
    }

    @Test func credentialImportBlocksAuthCompletionWithoutToken() {
        #expect(
            !DrawerSessionPolicy.shouldCompleteAuthAfterCredentialImport(
                flow: .createAccount,
                accountToken: nil
            )
        )
        #expect(
            !DrawerSessionPolicy.shouldCompleteAuthAfterCredentialImport(
                flow: .createAccount,
                accountToken: ""
            )
        )
        #expect(
            DrawerSessionPolicy.shouldCompleteAuthAfterCredentialImport(
                flow: .createAccount,
                accountToken: "token"
            )
        )
        #expect(
            !DrawerSessionPolicy.shouldCompleteAuthAfterCredentialImport(
                flow: .login,
                accountToken: nil
            )
        )
        #expect(
            DrawerSessionPolicy.shouldCompleteAuthAfterCredentialImport(
                flow: .login,
                accountToken: "token"
            )
        )
    }

    @Test func usableAccountTokenRejectsEmptyString() {
        #expect(!DrawerSessionPolicy.hasUsableAccountToken(nil))
        #expect(!DrawerSessionPolicy.hasUsableAccountToken(""))
        #expect(DrawerSessionPolicy.hasUsableAccountToken("abc"))
    }

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
