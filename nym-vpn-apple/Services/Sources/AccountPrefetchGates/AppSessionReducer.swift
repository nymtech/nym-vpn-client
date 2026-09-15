import Foundation

public enum NavigationIntent: Equatable, Sendable {
    case pushPlanPurchase
}

public enum SessionEvent: Equatable, Sendable {
    case authWillBegin(flow: AuthFlowKind, completesOnCredentialImport: Bool)
    case authHandoffCancelled
    case authDeeplinkProcessingStarted
    case authCompleted(outcome: AuthCompletionOutcome, flow: AuthFlowKind)
    case credentialRemoved
    case checkoutCompleted
    case checkoutDismissed
    case requestPlanPurchase
    case processingFinished
    case processingFailed(ProcessingFailure)
    case technicalOptInsContinued
}

public struct AppSessionContext: Equatable, Sendable {
    public var pendingAuthFlow: AuthFlowKind?
    public var authHandoffCompleted: Bool
    public var authHandoffCompletesOnCredentialImport: Bool
    public var lastAuthCompletionOutcome: AuthCompletionOutcome?
    public var pendingPlanPurchaseAfterOptIns: Bool
    public var isPurchaseFlowActive: Bool
    public var userDismissedCheckout: Bool

    public init(
        pendingAuthFlow: AuthFlowKind?,
        authHandoffCompleted: Bool,
        authHandoffCompletesOnCredentialImport: Bool,
        lastAuthCompletionOutcome: AuthCompletionOutcome?,
        pendingPlanPurchaseAfterOptIns: Bool,
        isPurchaseFlowActive: Bool,
        userDismissedCheckout: Bool = false
    ) {
        self.pendingAuthFlow = pendingAuthFlow
        self.authHandoffCompleted = authHandoffCompleted
        self.authHandoffCompletesOnCredentialImport = authHandoffCompletesOnCredentialImport
        self.lastAuthCompletionOutcome = lastAuthCompletionOutcome
        self.pendingPlanPurchaseAfterOptIns = pendingPlanPurchaseAfterOptIns
        self.isPurchaseFlowActive = isPurchaseFlowActive
        self.userDismissedCheckout = userDismissedCheckout
    }

    public static var initial: AppSessionContext {
        AppSessionContext(
            pendingAuthFlow: nil,
            authHandoffCompleted: false,
            authHandoffCompletesOnCredentialImport: false,
            lastAuthCompletionOutcome: nil,
            pendingPlanPurchaseAfterOptIns: false,
            isPurchaseFlowActive: false,
            userDismissedCheckout: false
        )
    }
}

public struct AppSessionEnvironment: Equatable, Sendable {
    public var isCredentialImported: Bool
    public var welcomeScreenDidDisplay: Bool
    public var isAccountActive: Bool
    public var processingKind: ProcessingFlowKind?
    public var accountSummaryLastFetchFailed: Bool
    public var validUntilIsFuture: Bool
    public var hasAccountSummary: Bool
    public var isAccountKnownInactive: Bool

    public init(
        isCredentialImported: Bool,
        welcomeScreenDidDisplay: Bool,
        isAccountActive: Bool,
        processingKind: ProcessingFlowKind? = nil,
        accountSummaryLastFetchFailed: Bool = false,
        validUntilIsFuture: Bool = false,
        hasAccountSummary: Bool = false,
        isAccountKnownInactive: Bool = false
    ) {
        self.isCredentialImported = isCredentialImported
        self.welcomeScreenDidDisplay = welcomeScreenDidDisplay
        self.isAccountActive = isAccountActive
        self.processingKind = processingKind
        self.accountSummaryLastFetchFailed = accountSummaryLastFetchFailed
        self.validUntilIsFuture = validUntilIsFuture
        self.hasAccountSummary = hasAccountSummary
        self.isAccountKnownInactive = isAccountKnownInactive
    }
}

public enum DrawerSessionCommand: Equatable, Sendable {
    case none
    case setWelcome
    case setOneClick
    case commitOneClick
    case setTechnicalOptIns
    case stageOneClickForCheckout
    case applyPostPurchaseDismissDestination
    case resetToWelcomeOnCredentialLoss
}

public struct AppSessionReducerResult: Equatable, Sendable {
    public var context: AppSessionContext
    public var drawerCommand: DrawerSessionCommand
    public var navigationIntent: NavigationIntent?
    public var showCheckoutDismissedFeedback: Bool
    public var cancelProcessing: Bool
    public var authRoute: AuthCompletionRoute?

    public init(
        context: AppSessionContext,
        drawerCommand: DrawerSessionCommand = .none,
        navigationIntent: NavigationIntent? = nil,
        showCheckoutDismissedFeedback: Bool = false,
        cancelProcessing: Bool = false,
        authRoute: AuthCompletionRoute? = nil
    ) {
        self.context = context
        self.drawerCommand = drawerCommand
        self.navigationIntent = navigationIntent
        self.showCheckoutDismissedFeedback = showCheckoutDismissedFeedback
        self.cancelProcessing = cancelProcessing
        self.authRoute = authRoute
    }
}

public enum AppSessionReducer: Equatable, Sendable {
    public static func reduce(
        context: AppSessionContext,
        environment: AppSessionEnvironment,
        event: SessionEvent
    ) -> AppSessionReducerResult {
        switch event {
        case let .authWillBegin(flow, completesOnCredentialImport):
            return authWillBegin(
                context: context,
                flow: flow,
                completesOnCredentialImport: completesOnCredentialImport
            )
        case .authHandoffCancelled:
            return authHandoffCancelled(context: context)
        case .authDeeplinkProcessingStarted:
            return authDeeplinkProcessingStarted(context: context)
        case let .authCompleted(outcome, flow):
            return authCompleted(context: context, outcome: outcome, flow: flow)
        case .credentialRemoved:
            return credentialRemoved(context: context)
        case .checkoutCompleted:
            return checkoutCompleted(context: context)
        case .checkoutDismissed:
            return checkoutDismissed(context: context, environment: environment)
        case .requestPlanPurchase:
            return requestPlanPurchase(context: context)
        case .processingFinished:
            return processingFinished(context: context, environment: environment)
        case .processingFailed:
            return processingFailed(context: context)
        case .technicalOptInsContinued:
            return technicalOptInsContinued(context: context, environment: environment)
        }
    }

    private static func authWillBegin(
        context: AppSessionContext,
        flow: AuthFlowKind,
        completesOnCredentialImport: Bool
    ) -> AppSessionReducerResult {
        var updated = context
        updated.pendingAuthFlow = flow
        updated.authHandoffCompleted = false
        updated.authHandoffCompletesOnCredentialImport = completesOnCredentialImport
        updated.lastAuthCompletionOutcome = nil
        return AppSessionReducerResult(context: updated)
    }

    private static func authHandoffCancelled(context: AppSessionContext) -> AppSessionReducerResult {
        var updated = context
        updated.pendingAuthFlow = nil
        updated.authHandoffCompleted = false
        updated.authHandoffCompletesOnCredentialImport = false
        updated.lastAuthCompletionOutcome = nil
        return AppSessionReducerResult(context: updated)
    }

    private static func authDeeplinkProcessingStarted(
        context: AppSessionContext
    ) -> AppSessionReducerResult {
        // The processing screen drives the login + routing on finish, so close the
        // handoff here without emitting an authRoute or drawer command. This makes
        // the imminent credential-import flip a no-op in DrawerCredentialImportPolicy.
        var updated = context
        updated.pendingAuthFlow = nil
        updated.authHandoffCompleted = true
        updated.authHandoffCompletesOnCredentialImport = false
        updated.lastAuthCompletionOutcome = nil
        return AppSessionReducerResult(context: updated)
    }

    private static func authCompleted(
        context: AppSessionContext,
        outcome: AuthCompletionOutcome,
        flow: AuthFlowKind
    ) -> AppSessionReducerResult {
        guard !context.authHandoffCompleted else {
            return AppSessionReducerResult(context: context)
        }
        var updated = context
        updated.authHandoffCompleted = true
        updated.pendingAuthFlow = nil
        updated.lastAuthCompletionOutcome = outcome
        let route = AuthCompletionRouter.route(outcome: outcome, flow: flow)
        return AppSessionReducerResult(context: updated, authRoute: route)
    }

    private static func credentialRemoved(context: AppSessionContext) -> AppSessionReducerResult {
        var updated = context
        updated.pendingAuthFlow = nil
        updated.authHandoffCompleted = false
        updated.authHandoffCompletesOnCredentialImport = false
        updated.isPurchaseFlowActive = false
        updated.userDismissedCheckout = false
        return AppSessionReducerResult(
            context: updated,
            drawerCommand: .resetToWelcomeOnCredentialLoss,
            cancelProcessing: true
        )
    }

    private static func checkoutCompleted(context: AppSessionContext) -> AppSessionReducerResult {
        var updated = context
        if CheckoutDismissPolicy.shouldClearDismissLedger(on: .checkoutCompleted) {
            updated.userDismissedCheckout = false
        }
        guard updated.isPurchaseFlowActive else {
            return AppSessionReducerResult(context: updated)
        }
        updated.isPurchaseFlowActive = false
        return AppSessionReducerResult(
            context: updated,
            drawerCommand: .commitOneClick
        )
    }

    private static func checkoutDismissed(
        context: AppSessionContext,
        environment: AppSessionEnvironment
    ) -> AppSessionReducerResult {
        guard context.isPurchaseFlowActive else {
            return AppSessionReducerResult(context: context)
        }
        var updated = context
        updated.isPurchaseFlowActive = false
        updated.userDismissedCheckout = true
        let showFeedback = IAPFeedbackPolicy.shouldShowCheckoutDismissedFeedback(
            isCredentialImported: environment.isCredentialImported,
            isAccountActive: environment.isAccountActive
        )
        return AppSessionReducerResult(
            context: updated,
            drawerCommand: .applyPostPurchaseDismissDestination,
            showCheckoutDismissedFeedback: showFeedback
        )
    }

    private static func requestPlanPurchase(context: AppSessionContext) -> AppSessionReducerResult {
        guard DrawerSessionPolicy.shouldBeginPlanPurchaseTransition(
            isPurchaseFlowActive: context.isPurchaseFlowActive
        ) else {
            return AppSessionReducerResult(context: context)
        }
        var updated = context
        updated.isPurchaseFlowActive = true
        if CheckoutDismissPolicy.shouldClearDismissLedger(on: .requestPlanPurchase) {
            updated.userDismissedCheckout = false
        }
        return AppSessionReducerResult(
            context: updated,
            drawerCommand: .stageOneClickForCheckout,
            navigationIntent: .pushPlanPurchase
        )
    }

    private static func processingFinished(
        context: AppSessionContext,
        environment: AppSessionEnvironment
    ) -> AppSessionReducerResult {
        let needsPurchase = !CheckoutDismissPolicy.shouldSuppressAutoPlanPurchase(
            userDismissedCheckout: context.userDismissedCheckout
        ) && DrawerSessionPolicy.shouldOfferPlanPurchaseAfterProcessing(
            processingKind: environment.processingKind,
            authOutcome: context.lastAuthCompletionOutcome,
            isAccountActive: environment.isAccountActive,
            accountSummaryLastFetchFailed: environment.accountSummaryLastFetchFailed,
            validUntilIsFuture: environment.validUntilIsFuture,
            hasAccountSummary: environment.hasAccountSummary,
            isAccountKnownInactive: environment.isAccountKnownInactive
        )

        var updated = context
        if !environment.welcomeScreenDidDisplay {
            updated.pendingPlanPurchaseAfterOptIns = needsPurchase
            return AppSessionReducerResult(
                context: updated,
                drawerCommand: .setTechnicalOptIns
            )
        }

        if needsPurchase {
            return requestPlanPurchase(context: updated)
        }
        return AppSessionReducerResult(
            context: updated,
            drawerCommand: .setOneClick
        )
    }

    private static func processingFailed(context: AppSessionContext) -> AppSessionReducerResult {
        // Un-stick the processing drawer the same way a dismissal does, and stop the
        // processing transition. The typed failure is surfaced to the user (snackbar)
        // by the coordinator; the reducer only restores a safe drawer destination.
        AppSessionReducerResult(
            context: context,
            drawerCommand: .applyPostPurchaseDismissDestination,
            cancelProcessing: true
        )
    }

    private static func technicalOptInsContinued(
        context: AppSessionContext,
        environment: AppSessionEnvironment
    ) -> AppSessionReducerResult {
        let purchaseAfter = context.pendingPlanPurchaseAfterOptIns
            && !CheckoutDismissPolicy.shouldSuppressAutoPlanPurchase(
                userDismissedCheckout: context.userDismissedCheckout
            )
        var updated = context
        updated.pendingPlanPurchaseAfterOptIns = false

        guard environment.isCredentialImported else {
            return AppSessionReducerResult(
                context: updated,
                drawerCommand: .setWelcome
            )
        }

        if purchaseAfter {
            return requestPlanPurchase(context: updated)
        }
        return AppSessionReducerResult(
            context: updated,
            drawerCommand: .setOneClick
        )
    }
}
