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

    var accountSummary: AccountSummary?
    var accountIdentifier: String?
    var deviceIdentifier: String?
    var accountSummaryFetchFailed = false

    @ObservationIgnored private var cancellables = Set<AnyCancellable>()

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
            pendingDrawerContent = .oneClick
            if purchaseAfter || !credentialsManager.isAccountActive() {
                onRequestPlanPurchase?()
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

    func handleCredentialChange(imported: Bool) {
        guard let current = drawerContent else { return }
        if imported {
            guard current.allowsCredentialPromotion else { return }
            startProcessingTransition()
        } else {
            pendingDrawerContent = nil
            cancelProcessingTransition()
            if current != .welcome {
                drawerContent = .welcome
            }
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
        let viewModel = ProcessingAccountViewModel(credentialsManager: credentialsManager)
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
        let needsPurchase = !credentialsManager.isAccountActive()

        if !appSettings.welcomeScreenDidDisplay {
            pendingPlanPurchaseAfterOptIns = needsPurchase
            pendingDrawerContent = .technicalOptIns
            return
        }

        pendingDrawerContent = .oneClick
        if needsPurchase {
            onRequestPlanPurchase?()
        }
    }
}
