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

    var drawerContent: AppDrawerContent?
    var pendingDrawerContent: AppDrawerContent?
    private(set) var processingViewModel: ProcessingAccountViewModel?

    @ObservationIgnored private var sessionContext = AppSessionContext.initial
    @ObservationIgnored private var planPurchaseTransitionTask: Task<Void, Never>?

    var accountSummary: AccountSummary?
    var accountIdentifier: String?
    var deviceIdentifier: String?
    var accountSummaryFetchFailed = false

    var purchaseTransitionOverlayVisible: Bool {
        DrawerSessionPolicy.showsPurchaseTransitionOverlay(
            isPurchaseFlowActive: sessionContext.isPurchaseFlowActive,
            isDrawerContentNil: drawerContent == nil
        )
    }

    @ObservationIgnored private var cancellables = Set<AnyCancellable>()
    @ObservationIgnored private var lastForegroundRefreshAt: Date?
    @ObservationIgnored private var pendingPostDisconnectAccountRefresh: Task<Void, Never>?
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
        self.connectionStatus = ConnectionStatusViewModel(connectionManager: connectionManager)
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
        self.connectionStatus = ConnectionStatusViewModel(connectionManager: connectionManager)
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

    public func handleSessionEvent(_ event: SessionEvent) {
        switch event {
        case .checkoutCompleted, .checkoutDismissed:
            cancelPlanPurchaseTransitionTask()
        case .processingFinished:
            processingDidFinish()
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
    }

    func consumeNavigationIntent() {
        navigationIntent = nil
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

    func leadingButtonTapped() {
        appSettings.currentAppearance = appSettings.currentAppearance == .dark ? .light : .dark
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

    func handleSceneBecameActive() {
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
                Task { @MainActor [weak self] in
                    guard let self else { return }
                    await self.ensureAccountRegisteredAfterCredentialImport(for: pendingFlow)
                    guard DrawerSessionPolicy.shouldCompleteAuthAfterCredentialImport(
                        flow: pendingFlow,
                        accountToken: self.credentialsManager.accountToken
                    ) else {
                        self.applyDrawerDestinationAfterPurchaseDismiss()
                        return
                    }
                    let outcome = await self.resolveAuthCompletionOutcome(for: pendingFlow)
                    self.handleAuthCompleted(outcome: outcome, flow: pendingFlow)
                }
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

    func dismissFamilyWarning() {
        oneClick.cancelGatewayIndependenceConsent()
        isFamilyWarningModalDisplayed = false
    }

    func requestPlanPurchaseTransition() {
        handleSessionEvent(.requestPlanPurchase)
    }

    func cancelPlanPurchaseTransitionTask() {
        planPurchaseTransitionTask?.cancel()
        planPurchaseTransitionTask = nil
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
            oneClick.requestIndependenceConsent()
            return
        }
        snackbarManager.enqueue(
            SnackbarItem(
                style: .critical,
                title: "connectionError.title".localizedString,
                message: ConnectionErrorCopy.message(reason: message),
                actionTitle: "disconnect".localizedString,
                onAction: { [weak self] in
                    self?.oneClick.connectButtonTapped()
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

    func startProcessingTransition(flow: ProcessingFlow) {
        let viewModel = ProcessingAccountViewModel(
            credentialsManager: credentialsManager,
            flow: flow
        )
        viewModel.sessionCoordinator = self
        processingViewModel = viewModel
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

    func viewModelProcessingKind(_ flow: ProcessingFlow) -> ProcessingFlowKind {
        switch flow {
        case .login:
            return .login
        case .postPurchase, .createAccount:
            return .postPurchase
        }
    }

    func applyDrawerDestinationAfterPurchaseDismiss() {
        switch DrawerSessionPolicy.drawerDestinationAfterPurchaseDismiss(
            isCredentialImported: appSettings.isCredentialImported,
            welcomeScreenDidDisplay: appSettings.welcomeScreenDidDisplay
        ) {
        case .welcome:
            drawerContent = .welcome
        case .technicalOptIns:
            pendingDrawerContent = .technicalOptIns
            drawerContent = .technicalOptIns
        case .oneClick:
            drawerContent = .oneClick
        }
    }

    func presentPurchaseDismissedFeedbackIfNeeded() {
        guard IAPFeedbackPolicy.shouldShowCheckoutDismissedFeedback(
            isCredentialImported: appSettings.isCredentialImported,
            isAccountActive: credentialsManager.isAccountActive()
        ) else { return }
        snackbarManager.enqueue(
            SnackbarItem(
                style: .warning,
                title: "purchasePlan.checkoutDismissed.title".localizedString,
                message: "purchasePlan.checkoutDismissed.message".localizedString,
                actionTitle: "oneClick.incompleteSubscription.action".localizedString,
                onAction: { [weak self] in
                    self?.requestPlanPurchaseTransition()
                },
                duration: 8
            )
        )
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

        if result.showCheckoutDismissedFeedback {
            presentPurchaseDismissedFeedbackIfNeeded()
        }

        switch result.drawerCommand {
        case .none:
            break
        case .setWelcome:
            pendingDrawerContent = .welcome
        case .setOneClick:
            pendingDrawerContent = .oneClick
            drawerContent = .oneClick
        case .commitOneClick:
            pendingDrawerContent = nil
            drawerContent = .oneClick
        case .setTechnicalOptIns:
            pendingDrawerContent = .technicalOptIns
        case .stageOneClickForCheckout:
            beginCheckoutDrawerTransition()
        case .applyPostPurchaseDismissDestination:
            pendingDrawerContent = nil
            applyDrawerDestinationAfterPurchaseDismiss()
        case .resetToWelcomeOnCredentialLoss:
            pendingDrawerContent = nil
            if drawerContent != .welcome {
                drawerContent = .welcome
            }
        }

        if result.navigationIntent == .pushPlanPurchase {
            navigationIntent = .pushPlanPurchase
            planPurchaseNavigationToken &+= 1
        }
    }

    func beginCheckoutDrawerTransition() {
        planPurchaseTransitionTask?.cancel()
        pendingDrawerContent = .oneClick
        planPurchaseTransitionTask = Task { @MainActor [weak self] in
            try? await Task.sleep(for: .milliseconds(Self.paywallDrawerDismissDelayMs))
            guard !Task.isCancelled, let self else { return }
            withAnimation(.easeInOut(duration: Self.paywallTransitionDuration)) {
                self.drawerContent = nil
            }
            self.planPurchaseTransitionTask = nil
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

    func reconcilePurchaseFlowAfterAccountRefresh() {
        guard DrawerSessionPolicy.shouldCompleteCheckoutAfterAccountRefresh(
            isPurchaseFlowActive: sessionContext.isPurchaseFlowActive,
            isAccountActive: credentialsManager.isAccountActive()
        ) else { return }
        handleSessionEvent(.checkoutCompleted)
    }
}
