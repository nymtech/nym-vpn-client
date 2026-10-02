import Combine
import Foundation
import SwiftUI
import SnackbarManager
import AccountPrefetchGates
import AppSettings
import ConnectionManager
import ConnectionTypes
import CredentialsManager
import GatewayManager
import ImpactGenerator
import NetworkMonitor
import Routes
import Settings
import TunnelStatus
#if os(macOS)
import GRPCManager
#endif

@Observable
@MainActor public final class AppFeatureViewModel: AppSessionCoordinating {
    public let appSettings: AppSettings
    public let credentialsManager: CredentialsManager
    public let snackbarManager: SnackbarManager
    public let connectionStatus: ConnectionStatusViewModel
    public let oneClick: OneClickViewModel

    public var path = NavigationPath()
    public private(set) var navigationIntent: NavigationIntent?
    public private(set) var planPurchaseNavigationToken: UInt = 0

    var isFamilyWarningModalDisplayed = false
    public private(set) var webSubscriptionPurchaseToken: UInt = 0

    var drawerContent: AppDrawerContent?
    var pendingDrawerContent: AppDrawerContent?
    private(set) var processingViewModel: ProcessingAccountViewModel?

    @ObservationIgnored private var sessionContext = AppSessionContext.initial
    @ObservationIgnored private var planPurchaseTransitionTask: Task<Void, Never>?
    @ObservationIgnored private var pendingPlanPurchaseNavigationAfterDrawerHide = false
    private(set) var isCheckoutNavigationPending = false

    var accountSummary: AccountSummary?
    var accountIdentifier: String?
    var deviceIdentifier: String?
    var accountSummaryFetchFailed = false

    var purchaseTransitionOverlayVisible: Bool {
        DrawerSessionPolicy.showsPurchaseTransitionOverlay(
            isPurchaseFlowActive: sessionContext.isPurchaseFlowActive,
            isDrawerContentNil: drawerContent == nil,
            isCheckoutNavigationPending: isCheckoutNavigationPending
        )
    }

    var shouldHideDrawerChromeDuringCheckout: Bool {
        PurchaseTransitionPolicy.shouldHideDrawerChromeDuringCheckout(
            isPurchaseFlowActive: sessionContext.isPurchaseFlowActive,
            isDrawerHidden: drawerContent == nil
        )
    }

    @ObservationIgnored private var cancellables = Set<AnyCancellable>()
    @ObservationIgnored private var reminderItemID: UUID?
    @ObservationIgnored private var reminderKey: String?
    @ObservationIgnored private var lastForegroundRefreshAt: Date?
    @ObservationIgnored private var pendingPostDisconnectAccountRefresh: Task<Void, Never>?
    @ObservationIgnored private var credentialImportCompletionTask: Task<Void, Never>?
    private static let foregroundRefreshMinInterval: TimeInterval = 300
    private static let postDisconnectAccountRefreshDelay: TimeInterval = 10

#if os(iOS)
    public init(
        appSettings: AppSettings,
        credentialsManager: CredentialsManager,
        connectionManager: ConnectionManager,
        gatewayManager: GatewayManager,
        snackbarManager: SnackbarManager,
        impactGenerator: ImpactGenerator,
        networkMonitor: NetworkMonitor
    ) {
        self.appSettings = appSettings
        self.credentialsManager = credentialsManager
        self.snackbarManager = snackbarManager
        self.connectionStatus = ConnectionStatusViewModel(connectionManager: connectionManager, networkMonitor: networkMonitor)
        self.oneClick = OneClickViewModel(
            appSettings: appSettings,
            connectionManager: connectionManager,
            credentialsManager: credentialsManager,
            gatewayManager: gatewayManager,
            snackbarManager: snackbarManager,
            impactGenerator: impactGenerator,
            networkMonitor: networkMonitor
        )

        finishInit()
    }
#elseif os(macOS)
    public init(
        appSettings: AppSettings,
        credentialsManager: CredentialsManager,
        connectionManager: ConnectionManager,
        gatewayManager: GatewayManager,
        snackbarManager: SnackbarManager,
        impactGenerator: ImpactGenerator,
        networkMonitor: NetworkMonitor,
        grpcManager: GRPCManager
    ) {
        self.appSettings = appSettings
        self.credentialsManager = credentialsManager
        self.snackbarManager = snackbarManager
        self.connectionStatus = ConnectionStatusViewModel(connectionManager: connectionManager, networkMonitor: networkMonitor)
        self.oneClick = OneClickViewModel(
            appSettings: appSettings,
            connectionManager: connectionManager,
            credentialsManager: credentialsManager,
            gatewayManager: gatewayManager,
            snackbarManager: snackbarManager,
            impactGenerator: impactGenerator,
            networkMonitor: networkMonitor,
            grpcManager: grpcManager
        )

        finishInit()
    }
#endif

    private func finishInit() {
        drawerContent = initialDrawerContent()
        accountSummary = credentialsManager.accountSummary
        accountIdentifier = credentialsManager.accountIdentifier
        deviceIdentifier = credentialsManager.deviceIdentifier
        accountSummaryFetchFailed = credentialsManager.accountSummaryLastFetchFailed

        observeAccountFields()
        wireConnectionStatusDelegates()
        oneClick.sessionCoordinator = self
    }

    public func handle(_ action: CoordinatorAction) {
        switch action {
        case .session(let event):
            handleSessionEvent(event)
        case .requestInactiveSubscriptionPurchase:
            requestInactiveSubscriptionPurchase()
        case .dismissPostPurchaseProcessing:
            requestDismissPostPurchaseProcessing()
        }
    }

    func handleSessionEvent(_ event: SessionEvent) {
        switch event {
        case .checkoutCompleted, .checkoutDismissed:
            cancelPendingPlanPurchaseNavigation()
        case .processingFinished:
            processingDidFinish()
            return
        case .processingFailed(let failure):
            processingDidFail(failure)
            return
        default:
            break
        }

        let result = AppSessionReducer.reduce(
            context: sessionContext,
            environment: makeSessionEnvironment(),
            event: event
        )
        applySessionResult(result)
        if let route = result.authRoute {
            applyAuthRoute(route)
        }

        if case let .authWillBegin(_, completesOnCredentialImport) = event,
           DrawerSessionPolicy.shouldBeginCredentialImportCompletionOnAuthWillBegin(
               completesOnCredentialImport: completesOnCredentialImport,
               isCredentialImported: appSettings.isCredentialImported,
               pendingAuthFlow: sessionContext.pendingAuthFlow,
               authHandoffCompleted: sessionContext.authHandoffCompleted
           ),
           let pendingFlow = sessionContext.pendingAuthFlow {
            beginCredentialImportCompletion(for: pendingFlow)
        }
    }

    func consumeNavigationIntent() {
        navigationIntent = nil
    }

    func checkoutNavigationDidComplete() {
        isCheckoutNavigationPending = false
    }

    var drawerTag: AppDrawerContent {
        pendingDrawerContent ?? drawerContent ?? .welcome
    }

    var drawerSlideID: AppDrawerSlideID {
        drawerTag.slideID
    }

    var shouldShowLogo: Bool {
        guard let content = pendingDrawerContent ?? drawerContent else { return false }
        return content.isOneClick
    }

    func technicalOptInsContinueTapped() {
        appSettings.welcomeScreenDidDisplay = true
        handleSessionEvent(.technicalOptInsContinued)
    }

    private func initialDrawerContent() -> AppDrawerContent {
        guard appSettings.isCredentialImported else { return .welcome }
        return appSettings.welcomeScreenDidDisplay ? .oneClick : .technicalOptIns
    }

    func drawerTransitionCompleted() {
        if let pending = pendingDrawerContent {
            drawerContent = pending
            pendingDrawerContent = nil
        }
        if drawerContent?.isProcessing == false {
            processingViewModel = nil
        }
    }

    /// Moves the drawer to a pre-auth state (`.welcome` / `.technicalOptIns`).
    ///
    /// `.welcome`, `.technicalOptIns` and `.processing` share one `slideID`
    /// (`.preauth`), so a transition among them doesn't change `drawerSlideID`.
    /// `DrawerView` only slides — and `drawerTransitionCompleted()`, the sole
    /// committer of `pendingDrawerContent` and the only place a finished
    /// `processingViewModel` is freed, only runs — on a slideID change. Staging
    /// a pre-auth state via `pendingDrawerContent` alone therefore never
    /// commits: `drawerTag` renders the staged state while `drawerContent`
    /// stays stale. When the slide identity won't change we commit directly (as
    /// `startProcessingTransition` does); otherwise stage and let the slide
    /// commit it, preserving the slide animation.
    func stagePreauthDrawer(_ content: AppDrawerContent) {
        // Compare against `drawerTag` (`pendingDrawerContent ?? drawerContent`) —
        // the exact value `drawerSlideID` observes — so this predicts whether a
        // slide will actually fire, leaving no guard/trigger divergence.
        guard (pendingDrawerContent ?? drawerContent ?? .welcome).slideID == content.slideID else {
            pendingDrawerContent = content
            return
        }
        pendingDrawerContent = nil
        if drawerContent?.isProcessing == true {
            processingViewModel = nil
        }
        drawerContent = content
    }

    func handleSceneBecameActive() {
        guard !connectionStatus.isConnectingLike, !isFamilyWarningModalDisplayed else { return }

        let now = Date()
        let shouldBypassThrottle = DrawerSessionPolicy.shouldBypassForegroundAccountRefreshThrottle(
            isPurchaseFlowActive: sessionContext.isPurchaseFlowActive,
            isAccountActive: credentialsManager.isAccountActive()
        )
        if !shouldBypassThrottle,
           let last = lastForegroundRefreshAt,
           now.timeIntervalSince(last) < Self.foregroundRefreshMinInterval {
            return
        }
        lastForegroundRefreshAt = now
        Task { [weak self] in
            guard let self else { return }
            await self.credentialsManager.updateAccountSummary(force: true)
            await MainActor.run {
                self.reconcilePurchaseFlowAfterAccountRefresh()
            }
        }
    }

    func handleTunnelStatusChange(from oldStatus: TunnelStatus, to newStatus: TunnelStatus) {
        guard oldStatus != newStatus else { return }

        if newStatus == .connecting || newStatus == .connected {
            pendingPostDisconnectAccountRefresh?.cancel()
            pendingPostDisconnectAccountRefresh = nil
            return
        }

        guard newStatus == .disconnected else { return }

        pendingPostDisconnectAccountRefresh?.cancel()
        pendingPostDisconnectAccountRefresh = Task { [weak self] in
            try? await Task.sleep(for: .seconds(Self.postDisconnectAccountRefreshDelay))
            guard !Task.isCancelled, let self else { return }
            await self.credentialsManager.updateAccountSummary(force: true)
            self.pendingPostDisconnectAccountRefresh = nil
        }
    }

    func noteAuthWillBegin(flow: AuthFlowKind, completesOnCredentialImport: Bool = false) {
        handleSessionEvent(.authWillBegin(flow: flow, completesOnCredentialImport: completesOnCredentialImport))
    }

    func noteAuthHandoffCancelled() {
        handleSessionEvent(.authHandoffCancelled)
    }

    func handleAuthCompleted(outcome: AuthCompletionOutcome, flow: AuthFlowKind) {
        handleSessionEvent(.authCompleted(outcome: outcome, flow: flow))
    }

    public func beginPrivyLoginProcessing(callbackURLString: String) {
        guard drawerContent?.isProcessing != true else { return }
        handleSessionEvent(.authDeeplinkProcessingStarted)
        startProcessingTransition(flow: .login, deeplinkLoginCallbackURL: callbackURLString)
    }

    func handleCredentialChange(imported: Bool) {
        if imported {
            let importAction = DrawerCredentialImportPolicy.action(
                imported: true,
                pendingAuthFlow: sessionContext.pendingAuthFlow,
                authHandoffCompleted: sessionContext.authHandoffCompleted,
                authHandoffCompletesOnCredentialImport: sessionContext.authHandoffCompletesOnCredentialImport,
                hasAccountToken: credentialsManager.accountToken != nil,
                drawerAllowsCredentialPromotion: drawerContent?.allowsCredentialPromotion ?? false
            )
            switch importAction {
            case .completeAuthOnImport(let pendingFlow):
                beginCredentialImportCompletion(for: pendingFlow)
            case .suppressDuringHandoff, .none:
                return
            case .startExternalProcessing:
                startProcessingTransition(flow: .postPurchase)
            }
        } else {
            handleSessionEvent(.credentialRemoved)
        }
    }

    func resolveAuthCompletionOutcome(for flow: AuthFlowKind) async -> AuthCompletionOutcome {
        await AuthCompletionOutcomeResolver.resolve(
            flow: flow,
            isAccountActive: { self.credentialsManager.isAccountActive() },
            updateAccountSummary: { untilActive in
                await self.credentialsManager.updateAccountSummary(
                    force: true,
                    untilActive: untilActive
                )
            }
        )
    }

    func confirmFamilyWarning() {
        isFamilyWarningModalDisplayed = false
        oneClick.independenceConsentAgreed()
    }

    func dismissFamilyWarning() {
        oneClick.cancelGatewayIndependenceConsent()
        isFamilyWarningModalDisplayed = false
    }

    func openNotificationSettingsFromFamilyWarning() {
        isFamilyWarningModalDisplayed = false
        path.append(HomeLink.settings)
        path.append(SettingLink.notifications)
    }

    func requestPlanPurchaseTransition() {
        handleSessionEvent(.requestPlanPurchase)
    }

    public func requestInactiveSubscriptionPurchase() {
#if os(iOS)
        requestPlanPurchaseTransition()
#elseif os(macOS)
        beginWebSubscriptionPurchase()
#endif
    }

    func beginWebSubscriptionPurchase() {
        webSubscriptionPurchaseToken &+= 1
    }

    public func reconcilePurchaseFlowAfterAccountRefresh() {
        guard !pendingPlanPurchaseNavigationAfterDrawerHide,
              planPurchaseTransitionTask == nil,
              !isCheckoutNavigationPending
        else { return }
        guard DrawerSessionPolicy.shouldCompleteCheckoutAfterAccountRefresh(
            isPurchaseFlowActive: sessionContext.isPurchaseFlowActive,
            isAccountActive: credentialsManager.isAccountActive()
        ) else { return }
        handleSessionEvent(.checkoutCompleted)
    }

    public func requestDismissPostPurchaseProcessing() {
        cancelProcessingTransition()
        cancelPendingPlanPurchaseNavigation()
        if PostPurchaseProcessingDismissPolicy.shouldRouteCheckoutDismissed(
            isPurchaseFlowActive: sessionContext.isPurchaseFlowActive
        ) {
            handleSessionEvent(.checkoutDismissed)
        } else {
            pendingDrawerContent = nil
            applyDrawerDestinationAfterPurchaseDismiss()
        }
    }

    func cancelPlanPurchaseTransitionTask() {
        planPurchaseTransitionTask?.cancel()
        planPurchaseTransitionTask = nil
        pendingPlanPurchaseNavigationAfterDrawerHide = false
        isCheckoutNavigationPending = false
    }

    func cancelPendingPlanPurchaseNavigation() {
        cancelPlanPurchaseTransitionTask()
        navigationIntent = nil
    }
}

private extension AppFeatureViewModel {
    static let paywallTransitionDuration = 0.35
    static let paywallDrawerDismissDelayMs = 150

    func wireConnectionStatusDelegates() {
        connectionStatus.onConnectionFailed = { [weak self] errorMessage in
            self?.presentConnectionFailedAlert(message: errorMessage)
        }
        connectionStatus.onConnectionStarted = { [weak self] in
            // Drop any in-flight connection-error snackbar when a fresh
            // attempt starts so we don't surface yesterday's failure.
            self?.snackbarManager.clear()
        }
    }

    func presentConnectionFailedAlert(message: String?) {
        let lastError = connectionStatus.connectionManager.lastError
        guard !ConnectionStatusViewModel.isNeedsRelaxedIndependenceCriteria(lastError)
        else {
            if appSettings.serverFamilyRemindersEnabled {
                isFamilyWarningModalDisplayed = true
            } else {
                connectionStatus.showsIndependenceWarning = true
                oneClick.independenceConsentAgreed()
            }
            return
        }
        snackbarManager.enqueue(
            SnackbarItem(
                style: .critical,
                title: "connectionError.title".localizedString,
                message: ConnectionErrorCopy.message(reason: message),
                actionTitle: "disconnect".localizedString,
                onAction: { [weak self] in
                    self?.oneClick.disconnectFromError()
                },
                duration: 7
            )
        )
    }

    func observeAccountFields() {
        credentialsManager.$accountSummary
            .receive(on: DispatchQueue.main)
            .sink { [weak self] _ in
                guard let self else { return }
                self.accountSummary = self.credentialsManager.accountSummary
                self.reconcilePurchaseFlowAfterAccountRefresh()
                self.updateExpiryReminder()
            }
            .store(in: &cancellables)

        credentialsManager.$accountIdentifier
            .receive(on: DispatchQueue.main)
            .sink { [weak self] in self?.accountIdentifier = $0 }
            .store(in: &cancellables)

        credentialsManager.$deviceIdentifier
            .receive(on: DispatchQueue.main)
            .sink { [weak self] in self?.deviceIdentifier = $0 }
            .store(in: &cancellables)

        credentialsManager.$accountSummaryLastFetchFailed
            .receive(on: DispatchQueue.main)
            .sink { [weak self] in self?.accountSummaryFetchFailed = $0 }
            .store(in: &cancellables)
    }

    /// Reconciles the upcoming-expiry renewal reminder against the current account
    /// state on every summary refresh: shows the right tier's sticky snackbar when
    /// one is due and not already shown/dismissed, and **retracts** a presented
    /// reminder once it is no longer justified (renewed elsewhere, logged out, tier
    /// changed). Reconciling — rather than firing once — is what lets it survive a
    /// `SnackbarManager.clear()` (connect/disconnect/network flaps call it): the next
    /// refresh re-enqueues because the item is no longer `current`.
    ///
    /// Subscriptions get "Renew now"; free passes get "Get a plan" — both route to
    /// the same purchase/renew entry. Suppression is per tier + expiry window.
    func updateExpiryReminder() {
        let desired = desiredExpiryReminder()

        // Retract a presented/queued reminder that no longer matches what's wanted.
        if let shownKey = reminderKey, shownKey != desired?.key, let id = reminderItemID {
            snackbarManager.dismiss(id: id)
            reminderItemID = nil
            reminderKey = nil
        }

        guard let desired else { return }

        // Explicitly dismissed by the user for this exact tier + expiry window.
        if appSettings.expiryReminderDismissedTier == desired.tier.rawValue,
           appSettings.expiryReminderDismissedValidUntil == desired.validUntil {
            return
        }
        // Still on screen for this key — nothing to do. (If it was cleared, `current`
        // no longer carries our id, so we fall through and re-enqueue.)
        if reminderKey == desired.key, let id = reminderItemID, snackbarManager.current?.id == id {
            return
        }

        let id = UUID()
        reminderItemID = id
        reminderKey = desired.key
        snackbarManager.enqueue(
            SnackbarItem(
                id: id,
                style: expiryReminderStyle(for: desired.tier),
                title: expiryReminderTitle(tier: desired.tier, isFreepass: desired.isFreepass),
                actionTitle: desired.isFreepass
                    ? "renewalReminder.action.getPlan".localizedString
                    : "renewalReminder.action.renew".localizedString,
                onAction: { [weak self] in
                    self?.requestInactiveSubscriptionPurchase()
                },
                onDismiss: { [weak self] in
                    guard let self else { return }
                    self.appSettings.expiryReminderDismissedTier = desired.tier.rawValue
                    self.appSettings.expiryReminderDismissedValidUntil = desired.validUntil
                },
                duration: nil
            )
        )
    }

    private struct DesiredExpiryReminder {
        let key: String
        let tier: ExpiryReminderTier
        let isFreepass: Bool
        let validUntil: Double
    }

    private func desiredExpiryReminder() -> DesiredExpiryReminder? {
        guard let summary = credentialsManager.accountSummary else { return nil }
        let tier = summary.expiryReminderTier()
        guard tier != .none else { return nil }
        let validUntil = summary.validUntilDate?.timeIntervalSince1970 ?? 0
        return DesiredExpiryReminder(
            key: "\(tier.rawValue)|\(validUntil)",
            tier: tier,
            isFreepass: summary.isFreepassPlan,
            validUntil: validUntil
        )
    }

    private func expiryReminderStyle(for tier: ExpiryReminderTier) -> SnackbarItem.Style {
        switch tier {
        case .hour24, .expired:
            return .critical
        default:
            return .warning
        }
    }

    private func expiryReminderTitle(tier: ExpiryReminderTier, isFreepass: Bool) -> String {
        let planSuffix = isFreepass ? "freepass" : "subscription"
        return "renewalReminder.\(tier.rawValue).\(planSuffix).title".localizedString
    }

    func applyAuthRoute(_ route: AuthCompletionRoute) {
        switch route {
        case .routeToPurchase:
            requestPlanPurchaseTransition()
        case .startProcessing(let kind):
            startProcessingTransition(flow: drawerProcessingFlow(for: kind))
        case .none:
            break
        }
    }

    func drawerProcessingFlow(for kind: ProcessingFlowKind) -> ProcessingFlow {
        switch kind {
        case .login:
            return .login
        case .postPurchase, .none:
            return .postPurchase
        }
    }

    func startProcessingTransition(flow: ProcessingFlow, deeplinkLoginCallbackURL: String? = nil) {
        let viewModel = ProcessingAccountViewModel(
            processing: credentialsManager,
            flow: flow,
            deeplinkLoginCallbackURL: deeplinkLoginCallbackURL
        )
        viewModel.sessionCoordinator = self
        processingViewModel = viewModel
        viewModel.start()
        // Welcome and processing share the same drawer slide identity, so
        // commit directly instead of staging through pendingDrawerContent —
        // that avoids triggering DrawerView.slideOut and lets the inner
        // ZStack animate the swap.
        pendingDrawerContent = nil
        drawerContent = .processing
    }

    func cancelProcessingTransition() {
        processingViewModel?.cancel()
        processingViewModel = nil
    }

    func processingDidFinish() {
        guard drawerContent?.isProcessing == true else { return }
        let processingKind = processingViewModel.map { viewModelProcessingKind($0.flow) }
        let result = AppSessionReducer.reduce(
            context: sessionContext,
            environment: makeSessionEnvironment(processingKind: processingKind),
            event: .processingFinished
        )
        applySessionResult(result)
    }

    func processingDidFail(_ failure: ProcessingFailure) {
        guard drawerContent?.isProcessing == true else { return }
        presentProcessingFailure(failure)
        let processingKind = processingViewModel.map { viewModelProcessingKind($0.flow) }
        let result = AppSessionReducer.reduce(
            context: sessionContext,
            environment: makeSessionEnvironment(processingKind: processingKind),
            event: .processingFailed(failure)
        )
        applySessionResult(result)
    }

    private func presentProcessingFailure(_ failure: ProcessingFailure) {
        let message: String
        switch failure {
        case .cancelled:
            return
        case .registration(let detail), .generic(let detail):
            message = detail
        }
        snackbarManager.enqueue(
            SnackbarItem(
                style: .critical,
                title: "error".localizedString,
                message: message
            )
        )
    }

    func viewModelProcessingKind(_ flow: ProcessingFlow) -> ProcessingFlowKind {
        switch flow {
        case .login:
            return .login
        case .postPurchase, .createAccount:
            return .postPurchase
        }
    }

    func applyDrawerDestinationAfterPurchaseDismiss() {
        applyPostPurchaseDrawerDestination(
            DrawerSessionPolicy.drawerDestinationAfterPurchaseDismiss(
                isCredentialImported: appSettings.isCredentialImported,
                welcomeScreenDidDisplay: appSettings.welcomeScreenDidDisplay
            )
        )
    }

    func applyDrawerDestinationAfterIncompleteCredentialImport() {
        if let destination = DrawerSessionPolicy.drawerDestinationAfterIncompleteCredentialImport(
            isCredentialImported: appSettings.isCredentialImported,
            welcomeScreenDidDisplay: appSettings.welcomeScreenDidDisplay
        ) {
            applyPostPurchaseDrawerDestination(destination)
            return
        }
        let authHandoffInProgress = sessionContext.pendingAuthFlow != nil
            && !sessionContext.authHandoffCompleted
        guard DrawerSessionPolicy.shouldRegressToWelcomeAfterImportFailure(
            isCredentialImported: appSettings.isCredentialImported,
            authHandoffInProgress: authHandoffInProgress
        ) else {
            return
        }
        applyDrawerDestinationAfterPurchaseDismiss()
    }

    func applyPostPurchaseDrawerDestination(_ destination: PostPurchaseDrawerDestination) {
        switch destination {
        case .welcome:
            drawerContent = .welcome
        case .technicalOptIns:
            pendingDrawerContent = .technicalOptIns
            drawerContent = .technicalOptIns
        case .oneClick:
            drawerContent = .oneClick
        }
    }

    func beginCredentialImportCompletion(for flow: AuthFlowKind) {
        guard !sessionContext.authHandoffCompleted else { return }
        credentialImportCompletionTask?.cancel()
        credentialImportCompletionTask = Task { @MainActor [weak self] in
            await self?.completeAuthOnImport(for: flow)
        }
    }

    func completeAuthOnImport(for flow: AuthFlowKind) async {
        await credentialsManager.ensureCredentialImportResolved()
        await ensureAccountRegisteredAfterCredentialImport(for: flow)
        guard DrawerSessionPolicy.shouldCompleteAuthAfterCredentialImport(
            flow: flow,
            accountToken: credentialsManager.accountToken
        ) else {
            applyDrawerDestinationAfterIncompleteCredentialImport()
            return
        }
        let outcome = await resolveAuthCompletionOutcome(for: flow)
        handleAuthCompleted(outcome: outcome, flow: flow)
    }

    func makeSessionEnvironment(processingKind: ProcessingFlowKind? = nil) -> AppSessionEnvironment {
        let summary = credentialsManager.accountSummary
        let resolvedProcessingKind = processingKind
            ?? processingViewModel.map { viewModelProcessingKind($0.flow) }
        return AppSessionEnvironment(
            isCredentialImported: appSettings.isCredentialImported,
            welcomeScreenDidDisplay: appSettings.welcomeScreenDidDisplay,
            isAccountActive: credentialsManager.isAccountActive(),
            processingKind: resolvedProcessingKind,
            accountSummaryLastFetchFailed: credentialsManager.accountSummaryLastFetchFailed,
            validUntilIsFuture: LoginSessionPolicy.validUntilIsFuture(
                validUntil: summary?.validUntilDate
            ),
            hasAccountSummary: summary != nil
        )
    }

    func applySessionResult(_ result: AppSessionReducerResult) {
        sessionContext = result.context

        if result.cancelProcessing {
            cancelProcessingTransition()
        }

        switch result.drawerCommand {
        case .none:
            break
        case .setWelcome:
            stagePreauthDrawer(.welcome)
        case .setOneClick:
            pendingDrawerContent = .oneClick
            drawerContent = .oneClick
        case .commitOneClick:
            pendingDrawerContent = nil
            drawerContent = .oneClick
        case .setTechnicalOptIns:
            stagePreauthDrawer(.technicalOptIns)
        case .stageOneClickForCheckout:
            beginCheckoutDrawerTransition(
                deferNavigationUntilDrawerHidden: result.navigationIntent == .pushPlanPurchase
            )
        case .applyPostPurchaseDismissDestination:
            pendingDrawerContent = nil
            applyDrawerDestinationAfterPurchaseDismiss()
        case .resetToWelcomeOnCredentialLoss:
            pendingDrawerContent = nil
            if drawerContent != .welcome {
                drawerContent = .welcome
            }
        }

        switch result.navigationIntent {
        case .pushPlanPurchase:
            navigationIntent = .pushPlanPurchase
            if result.drawerCommand != .stageOneClickForCheckout {
                planPurchaseNavigationToken &+= 1
            }
        default:
            break
        }
    }

    func beginCheckoutDrawerTransition(deferNavigationUntilDrawerHidden: Bool) {
        if deferNavigationUntilDrawerHidden {
            pendingPlanPurchaseNavigationAfterDrawerHide = true
        }
        let hadProcessingDrawer = drawerContent?.isProcessing == true
        planPurchaseTransitionTask?.cancel()
        planPurchaseTransitionTask = Task { @MainActor [weak self] in
            try? await Task.sleep(for: .milliseconds(Self.paywallDrawerDismissDelayMs))
            guard !Task.isCancelled, let self else { return }
            if self.drawerContent == nil {
                self.completeCheckoutDrawerTransition(hadProcessingDrawer: hadProcessingDrawer)
                return
            }
            withAnimation(.easeInOut(duration: Self.paywallTransitionDuration)) {
                self.drawerContent = nil
            }
            try? await Task.sleep(for: .seconds(Self.paywallTransitionDuration))
            guard !Task.isCancelled else { return }
            self.completeCheckoutDrawerTransition(hadProcessingDrawer: hadProcessingDrawer)
        }
    }

    func completeCheckoutDrawerTransition(hadProcessingDrawer: Bool = false) {
        let shouldMarkCheckoutNavigationPending = pendingPlanPurchaseNavigationAfterDrawerHide
        planPurchaseTransitionTask = nil
        if PurchaseTransitionPolicy.shouldCancelProcessingAfterDrawerHidden(
            hadProcessingDrawer: hadProcessingDrawer
        ) {
            cancelProcessingTransition()
        }
        guard shouldMarkCheckoutNavigationPending else { return }
        isCheckoutNavigationPending = true
        planPurchaseTransitionTask = Task { @MainActor [weak self] in
            try? await Task.sleep(
                for: .milliseconds(PurchaseTransitionPolicy.navigationPushDelayAfterDrawerHiddenMs)
            )
            guard !Task.isCancelled, let self else { return }
            self.pendingPlanPurchaseNavigationAfterDrawerHide = false
            self.planPurchaseNavigationToken &+= 1
        }
    }

    func ensureAccountRegisteredAfterCredentialImport(for flow: AuthFlowKind) async {
        guard DrawerSessionPolicy.shouldRegisterAccountAfterCredentialImport(
            flow: flow,
            accountToken: credentialsManager.accountToken
        ) else { return }
        do {
#if os(iOS)
            try await credentialsManager.performAccountRegistration()
#else
            try await credentialsManager.registerAccount()
#endif
        } catch {
            snackbarManager.enqueue(
                SnackbarItem(
                    style: .critical,
                    title: "error".localizedString,
                    message: error.localizedDescription
                )
            )
        }
    }
}
