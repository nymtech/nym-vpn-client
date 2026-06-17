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
@MainActor public final class AppFeatureViewModel {
    public let appSettings: AppSettings
    public let credentialsManager: CredentialsManager
    public let snackbarManager: SnackbarManager
    public let connectionStatus: ConnectionStatusViewModel
    public let oneClick: OneClickViewModel

    public var path = NavigationPath()

    var drawerContent: AppDrawerContent?
    var pendingDrawerContent: AppDrawerContent?
    private(set) var processingViewModel: ProcessingAccountViewModel?

    @ObservationIgnored private var pendingPlanPurchaseAfterOptIns = false
    @ObservationIgnored public var onRequestPlanPurchase: (() -> Void)?
    @ObservationIgnored private var pendingAuthFlow: AuthFlowKind?
    @ObservationIgnored private var authHandoffCompleted = false
    @ObservationIgnored private var authHandoffCompletesOnCredentialImport = false
    @ObservationIgnored private var isPurchaseFlowActive = false
    @ObservationIgnored private var planPurchaseTransitionTask: Task<Void, Never>?
    @ObservationIgnored private var lastAuthCompletionOutcome: AuthCompletionOutcome?

    var accountSummary: AccountSummary?
    var accountIdentifier: String?
    var deviceIdentifier: String?
    var accountSummaryFetchFailed = false

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
        let purchaseAfter = pendingPlanPurchaseAfterOptIns
        pendingPlanPurchaseAfterOptIns = false

        if appSettings.isCredentialImported {
            if purchaseAfter {
                requestPlanPurchaseTransition()
            } else {
                pendingDrawerContent = .oneClick
                drawerContent = .oneClick
            }
        } else {
            pendingDrawerContent = .welcome
        }
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
        if let last = lastForegroundRefreshAt,
           now.timeIntervalSince(last) < Self.foregroundRefreshMinInterval {
            return
        }
        lastForegroundRefreshAt = now
        Task { await credentialsManager.updateAccountSummary(force: true) }
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
        pendingAuthFlow = flow
        authHandoffCompleted = false
        authHandoffCompletesOnCredentialImport = completesOnCredentialImport
        lastAuthCompletionOutcome = nil
    }

    func noteAuthHandoffCancelled() {
        pendingAuthFlow = nil
        authHandoffCompleted = false
        authHandoffCompletesOnCredentialImport = false
        lastAuthCompletionOutcome = nil
    }

    func handleAuthCompleted(outcome: AuthCompletionOutcome, flow: AuthFlowKind) {
        guard !authHandoffCompleted else { return }
        authHandoffCompleted = true
        pendingAuthFlow = nil
        lastAuthCompletionOutcome = outcome
        routeAfterAuthCompletion(outcome: outcome, flow: flow)
    }

    func handleCredentialChange(imported: Bool) {
        guard let current = drawerContent else { return }
        if imported {
            let importAction = DrawerCredentialImportPolicy.action(
                imported: true,
                pendingAuthFlow: pendingAuthFlow,
                authHandoffCompleted: authHandoffCompleted,
                authHandoffCompletesOnCredentialImport: authHandoffCompletesOnCredentialImport,
                hasAccountToken: credentialsManager.accountToken != nil,
                drawerAllowsCredentialPromotion: current.allowsCredentialPromotion
            )
            switch importAction {
            case .completeAuthOnImport(let pendingFlow):
                Task { @MainActor [weak self] in
                    guard let self else { return }
                    let outcome = await self.resolveAuthCompletionOutcome(for: pendingFlow)
                    self.handleAuthCompleted(outcome: outcome, flow: pendingFlow)
                }
            case .suppressDuringHandoff, .none:
                return
            case .startExternalProcessing:
                startProcessingTransition(flow: .postPurchase)
            }
        } else {
            pendingAuthFlow = nil
            authHandoffCompleted = false
            authHandoffCompletesOnCredentialImport = false
            pendingDrawerContent = nil
            cancelProcessingTransition()
            if current != .welcome {
                drawerContent = .welcome
            }
        }
    }

    func resolveAuthCompletionOutcome(for flow: AuthFlowKind) async -> AuthCompletionOutcome {
        await AuthCompletionOutcomeResolver.resolve(
            flow: flow,
            isAccountActive: { credentialsManager.isAccountActive() },
            updateAccountSummary: { untilActive in
                await credentialsManager.updateAccountSummary(
                    force: true,
                    untilActive: untilActive
                )
            }
        )
    }

    func purchaseFlowDidComplete() {
        guard isPurchaseFlowActive else { return }
        cancelPlanPurchaseTransitionTask()
        isPurchaseFlowActive = false
        pendingDrawerContent = nil
        drawerContent = .oneClick
    }

    func purchaseFlowDidDismissWithoutComplete() {
        guard isPurchaseFlowActive else { return }
        cancelPlanPurchaseTransitionTask()
        isPurchaseFlowActive = false
        pendingDrawerContent = nil
        if appSettings.isCredentialImported {
            drawerContent = credentialsManager.isAccountActive() ? .oneClick : .welcome
        } else {
            drawerContent = .welcome
        }
    }

    func requestPlanPurchaseTransition() {
        guard DrawerSessionPolicy.shouldBeginPlanPurchaseTransition(
            isPurchaseFlowActive: isPurchaseFlowActive
        ) else {
            return
        }
        planPurchaseTransitionTask?.cancel()
        isPurchaseFlowActive = true
        pendingDrawerContent = .oneClick
        withAnimation(.easeInOut(duration: Self.paywallTransitionDuration)) {
            drawerContent = nil
        }
        planPurchaseTransitionTask = Task { @MainActor [weak self] in
            try? await Task.sleep(for: .milliseconds(Self.paywallTransitionDelayMs))
            guard !Task.isCancelled, let self else { return }
            self.onRequestPlanPurchase?()
            self.planPurchaseTransitionTask = nil
        }
    }

    func cancelPlanPurchaseTransitionTask() {
        planPurchaseTransitionTask?.cancel()
        planPurchaseTransitionTask = nil
    }
}

private extension AppFeatureViewModel {
    static let paywallTransitionDuration = 0.35
    static let paywallTransitionDelayMs = 350

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
            .sink { [weak self] in self?.accountSummary = $0 }
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

    func routeAfterAuthCompletion(outcome: AuthCompletionOutcome, flow: AuthFlowKind) {
        switch AuthCompletionRouter.route(outcome: outcome, flow: flow) {
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
        viewModel.onFinished = { [weak self] in
            self?.processingDidFinish()
        }
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
        let needsPurchase = DrawerSessionPolicy.shouldOfferPlanPurchaseAfterProcessing(
            processingKind: processingViewModel.map { viewModelProcessingKind($0.flow) },
            authOutcome: lastAuthCompletionOutcome,
            isAccountActive: credentialsManager.isAccountActive()
        )

        if !appSettings.welcomeScreenDidDisplay {
            pendingPlanPurchaseAfterOptIns = needsPurchase
            pendingDrawerContent = .technicalOptIns
            return
        }

        pendingDrawerContent = .oneClick
        if needsPurchase {
            requestPlanPurchaseTransition()
        } else {
            drawerContent = .oneClick
        }
    }

    func viewModelProcessingKind(_ flow: ProcessingFlow) -> ProcessingFlowKind {
        switch flow {
        case .login:
            return .login
        case .postPurchase, .createAccount:
            return .postPurchase
        }
    }
}
