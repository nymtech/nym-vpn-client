import Foundation
import Testing
import AccountPrefetchGates

struct AppSessionReducerTests {
    // E1: Privy cancel mid-handoff stays on welcome path with no processing.
    @Test func e1_privyCancel_clearsHandoffWithoutProcessing() {
        var context = AppSessionContext.initial
        let env = AppSessionEnvironment(
            isCredentialImported: false,
            welcomeScreenDidDisplay: false,
            isAccountActive: false
        )

        let afterBegin = AppSessionReducer.reduce(
            context: context,
            environment: env,
            event: .authWillBegin(flow: .createAccount, completesOnCredentialImport: true)
        )
        context = afterBegin.context
        #expect(context.pendingAuthFlow == .createAccount)
        #expect(context.authHandoffCompletesOnCredentialImport)
        #expect(!context.authHandoffCompleted)

        let afterCancel = AppSessionReducer.reduce(
            context: context,
            environment: env,
            event: .authHandoffCancelled
        )
        #expect(afterCancel.context.pendingAuthFlow == nil)
        #expect(!afterCancel.context.authHandoffCompletesOnCredentialImport)
        #expect(afterCancel.drawerCommand == .none)
        #expect(afterCancel.authRoute == nil)
        #expect(afterCancel.cancelProcessing == false)
    }

    // Deeplink processing takeover: closes the handoff so the imminent credential
    // import is a no-op, and emits no route/drawer command (the screen drives it).
    @Test func authDeeplinkProcessingStarted_closesHandoffWithoutRoute() {
        var context = AppSessionContext.initial
        let env = AppSessionEnvironment(
            isCredentialImported: false,
            welcomeScreenDidDisplay: false,
            isAccountActive: false
        )

        context = AppSessionReducer.reduce(
            context: context,
            environment: env,
            event: .authWillBegin(flow: .login, completesOnCredentialImport: true)
        ).context
        #expect(context.pendingAuthFlow == .login)

        let result = AppSessionReducer.reduce(
            context: context,
            environment: env,
            event: .authDeeplinkProcessingStarted
        )
        #expect(result.context.pendingAuthFlow == nil)
        #expect(result.context.authHandoffCompleted)
        #expect(!result.context.authHandoffCompletesOnCredentialImport)
        #expect(result.context.lastAuthCompletionOutcome == nil)
        #expect(result.drawerCommand == .none)
        #expect(result.authRoute == nil)
        #expect(result.cancelProcessing == false)
    }

    // E2: Privy success with inactive account routes to purchase.
    @Test func e2_privySuccessInactiveAccount_routesToPurchase() {
        let env = AppSessionEnvironment(
            isCredentialImported: true,
            welcomeScreenDidDisplay: true,
            isAccountActive: false
        )
        var context = AppSessionContext.initial
        context = AppSessionReducer.reduce(
            context: context,
            environment: env,
            event: .authWillBegin(flow: .createAccount, completesOnCredentialImport: true)
        ).context

        let result = AppSessionReducer.reduce(
            context: context,
            environment: env,
            event: .authCompleted(outcome: .registeredNeedsPurchase, flow: .createAccount)
        )

        #expect(result.context.authHandoffCompleted)
        #expect(result.context.lastAuthCompletionOutcome == .registeredNeedsPurchase)
        #expect(result.authRoute == .routeToPurchase)
    }

    @Test func e2_requestPlanPurchase_stagesCheckoutAndNavigationIntent() {
        let result = AppSessionReducer.reduce(
            context: .initial,
            environment: AppSessionEnvironment(
                isCredentialImported: true,
                welcomeScreenDidDisplay: true,
                isAccountActive: false
            ),
            event: .requestPlanPurchase
        )

        #expect(result.context.isPurchaseFlowActive)
        #expect(result.navigationIntent == .pushPlanPurchase)
        #expect(result.drawerCommand == .stageOneClickForCheckout)
    }

    // E3: IAP dismiss keeps imported inactive users on dashboard with feedback.
    @Test func e3_checkoutDismissedInactiveImported_dashboardWithFeedback() {
        var context = AppSessionContext.initial
        context.isPurchaseFlowActive = true

        let env = AppSessionEnvironment(
            isCredentialImported: true,
            welcomeScreenDidDisplay: true,
            isAccountActive: false
        )
        let result = AppSessionReducer.reduce(
            context: context,
            environment: env,
            event: .checkoutDismissed
        )

        #expect(!result.context.isPurchaseFlowActive)
        #expect(result.context.userDismissedCheckout)
        #expect(result.drawerCommand == .applyPostPurchaseDismissDestination)
        #expect(result.showCheckoutDismissedFeedback)
    }

    @Test func e3_postPurchaseDismissDestination_isOneClickWhenOptInsComplete() {
        let destination = DrawerSessionPolicy.drawerDestinationAfterPurchaseDismiss(
            isCredentialImported: true,
            welcomeScreenDidDisplay: true
        )
        #expect(destination == .oneClick)
    }

    // E4: IAP alert policy uses failed/cancel keys, not registration retry for purchase-only paths.
    @Test func e4_iapCancelRequiresUserAlert() {
        #expect(IAPFeedbackPolicy.requiresUserAlert(for: .userCancelled))
        #expect(IAPFeedbackPolicy.alertLocalizationKey(for: .userCancelled) == "purchasePlan.paymentCancelledAlert")
        #expect(!IAPFeedbackPolicy.showsRegistrationRetryOnAlert(isRegistrationFailure: false))
    }

    @Test func e4_iapFailedUsesFailedAlertKey() {
        #expect(IAPFeedbackPolicy.alertLocalizationKey(for: .failed) == "purchasePlan.paymentFailedAlert")
    }

    // E5: Login processing finishes on dashboard without purchase when account is active.
    @Test func e5_loginProcessing_finishesOnDashboardWithoutPurchase() {
        let env = AppSessionEnvironment(
            isCredentialImported: true,
            welcomeScreenDidDisplay: true,
            isAccountActive: true
        )

        let authResult = AppSessionReducer.reduce(
            context: .initial,
            environment: env,
            event: .authCompleted(outcome: .loginReady, flow: .login)
        )
        #expect(authResult.authRoute == .startProcessing(.login))

        var context = authResult.context
        let finishResult = AppSessionReducer.reduce(
            context: context,
            environment: AppSessionEnvironment(
                isCredentialImported: true,
                welcomeScreenDidDisplay: true,
                isAccountActive: true,
                processingKind: .login,
                accountSummaryLastFetchFailed: false,
                validUntilIsFuture: true,
                hasAccountSummary: true
            ),
            event: .processingFinished
        )

        #expect(finishResult.drawerCommand == .setOneClick)
        #expect(finishResult.navigationIntent == nil)
        #expect(!finishResult.context.isPurchaseFlowActive)
    }

    // Passphrase login: authWillBegin(completesOnCredentialImport: false) must still
    // complete via authCompleted and route to login processing (macOS parity).
    @Test func passphraseLoginHandoff_completesOnAuthCompleted() {
        var context = AppSessionContext.initial
        let env = AppSessionEnvironment(
            isCredentialImported: true,
            welcomeScreenDidDisplay: true,
            isAccountActive: true
        )

        context = AppSessionReducer.reduce(
            context: context,
            environment: env,
            event: .authWillBegin(flow: .login, completesOnCredentialImport: false)
        ).context
        #expect(context.pendingAuthFlow == .login)
        #expect(!context.authHandoffCompleted)

        let result = AppSessionReducer.reduce(
            context: context,
            environment: env,
            event: .authCompleted(outcome: .loginReady, flow: .login)
        )

        #expect(result.context.authHandoffCompleted)
        #expect(result.context.pendingAuthFlow == nil)
        #expect(result.authRoute == .startProcessing(.login))
    }

    @Test func passphraseLoginHandoff_inactiveAccountStartsLoginProcessing() {
        var context = AppSessionContext.initial
        let env = AppSessionEnvironment(
            isCredentialImported: true,
            welcomeScreenDidDisplay: true,
            isAccountActive: false
        )

        context = AppSessionReducer.reduce(
            context: context,
            environment: env,
            event: .authWillBegin(flow: .login, completesOnCredentialImport: false)
        ).context

        let result = AppSessionReducer.reduce(
            context: context,
            environment: env,
            event: .authCompleted(outcome: .registeredNeedsPurchase, flow: .login)
        )

        #expect(result.authRoute == .startProcessing(.login))
    }

    @Test func authCompleted_isIdempotentAfterHandoffCompletes() {
        var context = AppSessionContext.initial
        context.authHandoffCompleted = true

        let result = AppSessionReducer.reduce(
            context: context,
            environment: AppSessionEnvironment(
                isCredentialImported: true,
                welcomeScreenDidDisplay: true,
                isAccountActive: false
            ),
            event: .authCompleted(outcome: .registeredNeedsPurchase, flow: .createAccount)
        )

        #expect(result.authRoute == nil)
        #expect(result.drawerCommand == .none)
    }

    @Test func checkoutCompleted_noOpWhenPurchaseFlowInactive() {
        let result = AppSessionReducer.reduce(
            context: .initial,
            environment: AppSessionEnvironment(
                isCredentialImported: true,
                welcomeScreenDidDisplay: true,
                isAccountActive: false
            ),
            event: .checkoutCompleted
        )

        #expect(result.drawerCommand == .none)
    }

    @Test func credentialRemoved_cancelsProcessingAndResetsAuth() {
        var context = AppSessionContext.initial
        context.pendingAuthFlow = .login
        context.authHandoffCompleted = true

        let result = AppSessionReducer.reduce(
            context: context,
            environment: AppSessionEnvironment(
                isCredentialImported: false,
                welcomeScreenDidDisplay: false,
                isAccountActive: false
            ),
            event: .credentialRemoved
        )

        #expect(result.context.pendingAuthFlow == nil)
        #expect(!result.context.authHandoffCompleted)
        #expect(result.cancelProcessing)
        #expect(result.drawerCommand == .resetToWelcomeOnCredentialLoss)
    }

    @Test func checkoutCompleted_commitsOneClickDashboard() {
        var context = AppSessionContext.initial
        context.isPurchaseFlowActive = true

        let result = AppSessionReducer.reduce(
            context: context,
            environment: AppSessionEnvironment(
                isCredentialImported: true,
                welcomeScreenDidDisplay: true,
                isAccountActive: true
            ),
            event: .checkoutCompleted
        )

        #expect(!result.context.isPurchaseFlowActive)
        #expect(result.drawerCommand == .commitOneClick)
    }

    @Test func requestPlanPurchase_whenAlreadyActive_noOps() {
        var context = AppSessionContext.initial
        context.isPurchaseFlowActive = true

        let result = AppSessionReducer.reduce(
            context: context,
            environment: AppSessionEnvironment(
                isCredentialImported: true,
                welcomeScreenDidDisplay: true,
                isAccountActive: false
            ),
            event: .requestPlanPurchase
        )

        #expect(result.navigationIntent == nil)
        #expect(result.drawerCommand == .none)
        #expect(result.context.isPurchaseFlowActive)
    }

    @Test func e2_authCompletedThenRequestPurchase_stagesSingleCheckout() {
        let env = AppSessionEnvironment(
            isCredentialImported: true,
            welcomeScreenDidDisplay: true,
            isAccountActive: false
        )
        let authResult = AppSessionReducer.reduce(
            context: .initial,
            environment: env,
            event: .authCompleted(outcome: .registeredNeedsPurchase, flow: .createAccount)
        )
        #expect(authResult.authRoute == .routeToPurchase)

        let purchaseResult = AppSessionReducer.reduce(
            context: authResult.context,
            environment: env,
            event: .requestPlanPurchase
        )
        #expect(purchaseResult.context.isPurchaseFlowActive)
        #expect(purchaseResult.navigationIntent == .pushPlanPurchase)
        #expect(purchaseResult.drawerCommand == .stageOneClickForCheckout)

        let repeatResult = AppSessionReducer.reduce(
            context: purchaseResult.context,
            environment: env,
            event: .requestPlanPurchase
        )
        #expect(repeatResult.navigationIntent == nil)
    }

    @Test func processingFinished_firstLaunch_stagesTechnicalOptInsWithPurchasePending() {
        var context = AppSessionContext.initial
        context.lastAuthCompletionOutcome = .registeredNeedsPurchase

        let result = AppSessionReducer.reduce(
            context: context,
            environment: AppSessionEnvironment(
                isCredentialImported: true,
                welcomeScreenDidDisplay: false,
                isAccountActive: false,
                processingKind: .postPurchase,
                accountSummaryLastFetchFailed: false,
                validUntilIsFuture: false,
                hasAccountSummary: true
            ),
            event: .processingFinished
        )

        #expect(result.drawerCommand == .setTechnicalOptIns)
        #expect(result.context.pendingPlanPurchaseAfterOptIns)
        #expect(result.navigationIntent == nil)
    }

    @Test func processingFinished_firstLaunch_skipsPurchaseWhenActive() {
        var context = AppSessionContext.initial
        context.lastAuthCompletionOutcome = .registeredActive

        let result = AppSessionReducer.reduce(
            context: context,
            environment: AppSessionEnvironment(
                isCredentialImported: true,
                welcomeScreenDidDisplay: false,
                isAccountActive: true,
                processingKind: .postPurchase,
                accountSummaryLastFetchFailed: false,
                validUntilIsFuture: true,
                hasAccountSummary: true
            ),
            event: .processingFinished
        )

        #expect(result.drawerCommand == .setTechnicalOptIns)
        #expect(!result.context.pendingPlanPurchaseAfterOptIns)
    }

    @Test func processingFinished_inactiveAfterOptIns_routesToPurchase() {
        var context = AppSessionContext.initial
        context.lastAuthCompletionOutcome = .registeredNeedsPurchase

        let result = AppSessionReducer.reduce(
            context: context,
            environment: AppSessionEnvironment(
                isCredentialImported: true,
                welcomeScreenDidDisplay: true,
                isAccountActive: false,
                processingKind: .postPurchase,
                accountSummaryLastFetchFailed: false,
                validUntilIsFuture: false,
                hasAccountSummary: true
            ),
            event: .processingFinished
        )

        #expect(result.navigationIntent == .pushPlanPurchase)
        #expect(result.drawerCommand == .stageOneClickForCheckout)
        #expect(result.context.isPurchaseFlowActive)
    }

    @Test func technicalOptInsContinued_withPendingPurchase_routesToCheckout() {
        var context = AppSessionContext.initial
        context.pendingPlanPurchaseAfterOptIns = true

        let result = AppSessionReducer.reduce(
            context: context,
            environment: AppSessionEnvironment(
                isCredentialImported: true,
                welcomeScreenDidDisplay: true,
                isAccountActive: false
            ),
            event: .technicalOptInsContinued
        )

        #expect(!result.context.pendingPlanPurchaseAfterOptIns)
        #expect(result.navigationIntent == .pushPlanPurchase)
        #expect(result.drawerCommand == .stageOneClickForCheckout)
    }

    @Test func technicalOptInsContinued_withoutCredentials_returnsWelcome() {
        var context = AppSessionContext.initial

        let result = AppSessionReducer.reduce(
            context: context,
            environment: AppSessionEnvironment(
                isCredentialImported: false,
                welcomeScreenDidDisplay: true,
                isAccountActive: false
            ),
            event: .technicalOptInsContinued
        )

        #expect(result.drawerCommand == .setWelcome)
    }

    @Test func checkoutDismissed_noOpWhenPurchaseFlowInactive() {
        let result = AppSessionReducer.reduce(
            context: .initial,
            environment: AppSessionEnvironment(
                isCredentialImported: true,
                welcomeScreenDidDisplay: true,
                isAccountActive: false
            ),
            event: .checkoutDismissed
        )

        #expect(result.drawerCommand == .none)
        #expect(!result.showCheckoutDismissedFeedback)
    }

    @Test func checkoutDismissed_activeAccount_skipsDismissedFeedback() {
        var context = AppSessionContext.initial
        context.isPurchaseFlowActive = true

        let result = AppSessionReducer.reduce(
            context: context,
            environment: AppSessionEnvironment(
                isCredentialImported: true,
                welcomeScreenDidDisplay: true,
                isAccountActive: true
            ),
            event: .checkoutDismissed
        )

        #expect(result.drawerCommand == .applyPostPurchaseDismissDestination)
        #expect(!result.showCheckoutDismissedFeedback)
    }

    @Test func credentialRemoved_clearsActivePurchaseFlow() {
        var context = AppSessionContext.initial
        context.isPurchaseFlowActive = true

        let result = AppSessionReducer.reduce(
            context: context,
            environment: AppSessionEnvironment(
                isCredentialImported: false,
                welcomeScreenDidDisplay: false,
                isAccountActive: false
            ),
            event: .credentialRemoved
        )

        #expect(!result.context.isPurchaseFlowActive)
    }

    @Test func processingFinished_activeAccountWithStaleNeedsPurchaseRoutesToDashboard() {
        var context = AppSessionContext.initial
        context.lastAuthCompletionOutcome = .registeredNeedsPurchase

        let result = AppSessionReducer.reduce(
            context: context,
            environment: AppSessionEnvironment(
                isCredentialImported: true,
                welcomeScreenDidDisplay: true,
                isAccountActive: true,
                processingKind: .postPurchase
            ),
            event: .processingFinished
        )

        #expect(result.drawerCommand == .setOneClick)
        #expect(result.navigationIntent == nil)
    }

    @Test func processingFinished_skipsAutoPurchaseAfterUserDismissedCheckout() {
        var context = AppSessionContext.initial
        context.lastAuthCompletionOutcome = .registeredNeedsPurchase
        context.userDismissedCheckout = true

        let result = AppSessionReducer.reduce(
            context: context,
            environment: AppSessionEnvironment(
                isCredentialImported: true,
                welcomeScreenDidDisplay: true,
                isAccountActive: false,
                processingKind: .postPurchase
            ),
            event: .processingFinished
        )

        #expect(result.drawerCommand == .setOneClick)
        #expect(result.navigationIntent == nil)
    }

    @Test func checkoutDismissed_setsUserDismissedCheckoutLedger() {
        var context = AppSessionContext.initial
        context.isPurchaseFlowActive = true

        let result = AppSessionReducer.reduce(
            context: context,
            environment: AppSessionEnvironment(
                isCredentialImported: true,
                welcomeScreenDidDisplay: true,
                isAccountActive: false
            ),
            event: .checkoutDismissed
        )

        #expect(result.context.userDismissedCheckout)
    }

    @Test func processingFailed_cancelsProcessingAndUnsticksDrawer() {
        let context = AppSessionContext.initial

        let result = AppSessionReducer.reduce(
            context: context,
            environment: AppSessionEnvironment(
                isCredentialImported: true,
                welcomeScreenDidDisplay: true,
                isAccountActive: false
            ),
            event: .processingFailed(.registration("boom"))
        )

        #expect(result.cancelProcessing)
        #expect(result.drawerCommand == .applyPostPurchaseDismissDestination)
        #expect(result.navigationIntent == nil)
        #expect(result.context == context)
    }
}
