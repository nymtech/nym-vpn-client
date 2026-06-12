import Combine
import Foundation
import SwiftUI
import SnackbarManager
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
@MainActor public final class AppFeatureViewModel {
    public let appSettings: AppSettings
    public let credentialsManager: CredentialsManager
    public let snackbarManager: SnackbarManager
    public let connectionStatus: ConnectionStatusViewModel
    public let oneClick: OneClickViewModel
    private let connectionManager: ConnectionManager

    public var path = NavigationPath()

    var drawerContent: AppDrawerContent?
    var pendingDrawerContent: AppDrawerContent?
    private(set) var processingViewModel: ProcessingAccountViewModel?

    @ObservationIgnored private var pendingPlanPurchaseAfterOptIns = false
    @ObservationIgnored var pendingProcessingFlow: ProcessingFlow = .createAccount
    let onboardingSession = OnboardingSession.shared

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
        self.connectionManager = connectionManager
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
        self.connectionManager = connectionManager
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
            pendingDrawerContent = .oneClick
            if purchaseAfter {
                presentPlanPurchaseFlow()
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

    func handleCredentialChange(imported: Bool) {
        guard let current = drawerContent else { return }
        if imported {
            return
        }
        pendingDrawerContent = nil
        cancelProcessingTransition()
        if current != .welcome {
            drawerContent = .welcome
        }
    }

    func handleAuthRegistrationComplete() {
        guard drawerContent?.allowsCredentialPromotion == true else { return }
        guard onboardingSession.canStartProcessing else { return }
        onboardingSession.advance(to: .registered)
        startProcessingTransition()
    }

    func requestPlanPurchaseIfNeeded() -> Bool {
        guard !credentialsManager.isAccountActive() else { return false }
        guard onboardingSession.shouldPresentPurchase else { return false }
        onboardingSession.markPurchaseFlowPresented()
        return true
    }

    func presentPlanPurchaseFlow() {
        guard requestPlanPurchaseIfNeeded() else { return }
        Task { @MainActor [weak self] in
            await Task.yield()
            guard let self else { return }
            self.path.append(HomeLink.settings)
            self.path.append(SettingLink.generatePassphrase(displayPurchaseView: true))
        }
    }
}

private extension AppFeatureViewModel {
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

    func startProcessingTransition() {
        let viewModel = ProcessingAccountViewModel(
            credentialsManager: credentialsManager,
            flow: pendingProcessingFlow,
            canPrefetchZkNyms: { [weak self] in
                self?.connectionManager.canPrefetchZkNymsFromApp ?? true
            }
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
        onboardingSession.advance(to: .processingComplete)
        let needsPurchase = !credentialsManager.isAccountActive()

        if !appSettings.welcomeScreenDidDisplay {
            pendingPlanPurchaseAfterOptIns = needsPurchase
            pendingDrawerContent = .technicalOptIns
            return
        }

        pendingDrawerContent = .oneClick
        if needsPurchase {
            presentPlanPurchaseFlow()
        } else {
            onboardingSession.advance(to: .finished)
        }
    }

}
