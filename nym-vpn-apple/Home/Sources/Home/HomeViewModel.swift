import Combine
import SwiftUI
import AppSettings
import ConnectionManager
import Constants
import ConfigurationManager
import Device
import ConnectionTypes
import CredentialsManager
import ExternalLinkManager
import GatewayManager
import MessagesManager
import MessageModels
import NetworkMonitor
import Routes
import Settings
import TunnelMixnet
import TunnelStatus
import Tunnels
import UIComponents
import ImpactGenerator
#if os(macOS)
import GRPCManager
#endif

@Observable
@MainActor public class HomeViewModel: HomeFlowState {
    let title = "NymVPN".localizedString
    let connectToLocalizedTitle = "connectTo".localizedString
    let networkSelectLocalizedTitle = "selectNetwork".localizedString

    let appSettings: AppSettings
    let configurationManager: ConfigurationManager
    let credentialsManager: CredentialsManager
    let externalLinkManager: ExternalLinkManager
    let gatewayManager: GatewayManager
    let networkMonitor: NetworkMonitor
    let impactGenerator: ImpactGenerator
#if os(macOS)
    let grpcManager: GRPCManager
#endif
    let messagesManager: MessagesManager
    let anonymousButtonViewModel = NetworkButtonViewModel(
        type: .mixnet5hop,
        appSettings: .shared,
        connectionManager: .shared
    )
    let fastButtonViewModel = NetworkButtonViewModel(
        type: .wireguard,
        appSettings: .shared,
        connectionManager: .shared
    )

    var connectionManager: ConnectionManager
    var cancellables = Set<AnyCancellable>()
    var tunnelStatusUpdateCancellable: AnyCancellable?
    var tunnelRetryAttemptCancellable: AnyCancellable?
    var tunnelConnectingStateCancellable: AnyCancellable?
    var lastTunnelStatus = TunnelStatus.disconnected
    var lastError: Error?

#if os(macOS)
    public var isServing = false
#endif

    var activeTunnel: Tunnel?
    var statusButtonConfig = StatusButtonConfig.disconnected

    /// Use updateStatusInfoState(with:) to update the statusInfoState
    var statusInfoState = StatusInfoState.initialising

    /// Info from connecting/connected data, current gatewayId, that tunnel is connecting/connected to
    var connectionInfoData: ConnectionInfoData?

    var connectButtonState = ConnectButtonState.connect
    var isModeInfoOverlayDisplayed = false
    var isOfflineOverlayDisplayed = false
    var isUpdateAvailableOverlayDisplayed = false
    var isStatisticsOverlayDisplayed = false
    var snackBarMessage: SnackBarMessage?
    var isSnackBarDisplayed = false {
        didSet {
            Task {
                try? await Task.sleep(for: .seconds(1))
                guard !isSnackBarDisplayed else { return }
                messagesManager.messageDidClose()
            }
        }
    }

    var offlineOverlayConfiguration: ActionDialogConfiguration {
        ActionDialogConfiguration(
            systemIconImageName: "exclamationmark.circle",
            titleLocalizedString: "home.modal.noInternetConnection.title".localizedString,
            subtitleLocalizedString: "home.modal.noInternetConnection.subtitle".localizedString,
            yesLocalizedString: "close".localizedString
        )
    }

    var updateAvailableOverlayConfiguration: ActionDialogConfiguration {
        ActionDialogConfiguration(
            systemIconImageName: "exclamationmark.circle",
            titleLocalizedString: "home.modal.updateAvailable.title".localizedString,
            subtitleLocalizedString: "home.modal.updateAvailable.subtitle".localizedString,
            yesLocalizedString: "home.modal.update".localizedString,
            yesAction: {
                try? ExternalLinkManager.shared.openExternalURL(urlString: Constants.downloadLink.rawValue)
            }
        )
    }

#if os(iOS)
    public init(
        appSettings: AppSettings,
        connectionManager: ConnectionManager,
        configurationManager: ConfigurationManager,
        credentialsManager: CredentialsManager,
        networkMonitor: NetworkMonitor,
        externalLinkManager: ExternalLinkManager,
        gatewayManager: GatewayManager,
        impactGenerator: ImpactGenerator,
        messagesManager: MessagesManager
    ) {
        self.appSettings = appSettings
        self.connectionManager = connectionManager
        self.configurationManager = configurationManager
        self.credentialsManager = credentialsManager
        self.externalLinkManager = externalLinkManager
        self.gatewayManager = gatewayManager
        self.impactGenerator = impactGenerator
        self.networkMonitor = networkMonitor
        self.messagesManager = messagesManager
        super.init()

        setup()
    }
#elseif os(macOS)
    public init(
        appSettings: AppSettings,
        connectionManager: ConnectionManager,
        configurationManager: ConfigurationManager,
        credentialsManager: CredentialsManager,
        networkMonitor: NetworkMonitor,
        grpcManager: GRPCManager,
        externalLinkManager: ExternalLinkManager,
        gatewayManager: GatewayManager,
        impactGenerator: ImpactGenerator,
        messagesManager: MessagesManager
    ) {
        self.appSettings = appSettings
        self.connectionManager = connectionManager
        self.configurationManager = configurationManager
        self.credentialsManager = credentialsManager
        self.networkMonitor = networkMonitor
        self.grpcManager = grpcManager
        self.externalLinkManager = externalLinkManager
        self.impactGenerator = impactGenerator
        self.gatewayManager = gatewayManager
        self.messagesManager = messagesManager
        super.init()

        setup()
    }
#endif
}

// MARK: - Navigation -

public extension HomeViewModel {
    @MainActor func navigateToSettings() {
        impactGenerator.softImpact()
        path.append(HomeLink.settings)
    }

    @MainActor func navigateToEntryGateways() {
        impactGenerator.softImpact()
        path.append(HomeLink.entryGateways)
    }

    @MainActor func navigateToExitGateways() {
        impactGenerator.softImpact()
        path.append(HomeLink.exitGateways)
    }

    @MainActor func navigateToGatewayDetails(for hopType: HopType, gatewayType: NodeType) {
        let gateway: GatewayNode?
        switch hopType {
        case .entry:
            let entryGatewayId = connectionInfoData?.entryGatewayId ?? connectionManager.entryGateway.gatewayId
            gateway = gatewayManager.gateway(with: entryGatewayId, gatewayType: gatewayType)
        case .exit:
            let exitGatewayId = connectionInfoData?.exitGatewayId ?? connectionManager.exitRouter.gatewayId
            gateway = gatewayManager.gateway(with: exitGatewayId, gatewayType: gatewayType)
        }
        guard let gateway else { return }
        navigateToGatewayDetails(gateway: gateway, hopType: hopType)
    }

    @MainActor private func navigateToGatewayDetails(gateway: GatewayNode, hopType: HopType) {
        path.append(HomeLink.gatewayDetails(gateway: gateway, hopType: hopType))
    }

    @MainActor func navigateToOnboarding() {
        path.append(HomeLink.onboarding)
    }

    @MainActor func navigateToPlanPurchase() {
        path.append(HomeLink.settings)
        path.append(SettingLink.generatePassphrase(displayPurchaseView: true))
    }

#if os(macOS)
    func navigateToDaemonEnable() {
        path.append(HomeLink.settings)
        path.append(SettingLink.daemonEnable)
    }
#endif

    func checkExpiryBanner() {
        guard let accountSummary = credentialsManager.accountSummary else { return }
        let now = Date()

        var shouldShow = false
        if accountSummary.isExpiringSoon {
            let lastDismissed = Date(timeIntervalSince1970: appSettings.expirySoonDismissedAt)
            guard now.timeIntervalSince(lastDismissed) > 86400 else { return }
            shouldShow = true
        } else if accountSummary.isExpiringWarning {
            guard appSettings.expiryWarningDismissedAt == 0 else { return }
            shouldShow = true
        }

        guard shouldShow else { return }
        let dateText = accountSummary.formattedValidUntilDate ?? "-"
        messagesManager.enqueueExpiryBanner(
            subtitle: dateText,
            ctaAction: { [weak self] in
                self?.navigateToPlanPurchase()
            },
            closeAction: { [weak self] in
                self?.dismissExpiryBanner()
            }
        )
    }

    func dismissExpiryBanner() {
        guard let accountSummary = credentialsManager.accountSummary else { return }
        let now = Date().timeIntervalSince1970

        if accountSummary.isExpiringSoon {
            appSettings.expirySoonDismissedAt = now
        } else if accountSummary.isExpiringWarning {
            appSettings.expiryWarningDismissedAt = now
        }
    }
}

// MARK: - Configuration -
private extension HomeViewModel {
    func setup() {
        setupTunnelManagerObservers()
        setupUpdateRequiredObserver()
        setupSystemMessageObservers()
        setupIsMnemonicImportedObserver()
        setupAccountSummaryObserver()
        setupSubscriptionPaymentObserver()
        configurePassphraseBanner()
#if os(iOS)
        setupConnectionErrorObservers()
        setupNetworkMonitorObservers()
#elseif os(macOS)
        setupGRPCManagerObservers()
#endif

    }

    func configurePassphraseBanner() {
        guard !Device.isMacOS else { return }
        messagesManager.configurePassphraseBanner(
            ctaAction: { [weak self] in
                self?.path.append(HomeLink.settings)
                self?.path.append(SettingLink.passphrase)
            }
        )
    }

    func setupTunnelManagerObservers() {
        connectionManager.$isTunnelManagerLoaded.sink { [weak self] result in
            switch result {
            case .success, .none:
                self?.resetStatusInfoState()
            case let .failure(error):
                self?.updateStatusInfoState(with: .error(message: error.localizedDescription))
            }
        }
        .store(in: &cancellables)
#if os(iOS)
        connectionManager.$activeTunnel
            .receive(on: DispatchQueue.main)
            .sink { [weak self] tunnel in
                guard let tunnel, let self else { return }
                MainActor.assumeIsolated {
                    self.activeTunnel = tunnel
                    self.configureTunnelStatusObservation(with: tunnel)
                }
            }
            .store(in: &cancellables)
#endif
    }

    func setupUpdateRequiredObserver() {
        configurationManager.$isCurrentAppVersionCompatible
            .receive(on: DispatchQueue.main)
            .sink { [weak self] value in
                guard !value else { return }
                MainActor.assumeIsolated {
                    self?.isUpdateAvailableOverlayDisplayed = !value
                }
            }
            .store(in: &cancellables)
    }

    func setupAccountSummaryObserver() {
        credentialsManager.$accountSummary
            .receive(on: DispatchQueue.main)
            .sink { [weak self] _ in
                MainActor.assumeIsolated {
                    self?.checkExpiryBanner()
                    self?.updateConnectButtonStateForSubscription()
                }
            }
            .store(in: &cancellables)
    }

    func setupSubscriptionPaymentObserver() {
        credentialsManager.$didReceiveSubscriptionPayment
            .receive(on: DispatchQueue.main)
            .sink { [weak self] didReceive in
                guard didReceive else { return }
                MainActor.assumeIsolated {
                    self?.credentialsManager.didReceiveSubscriptionPayment = false
                    self?.path = .init()
                    self?.messagesManager.addAndProcess(
                        SnackBarMessage(
                            text: "subscriptionPayment.received".localizedString,
                            style: .info
                        )
                    )
                }
            }
            .store(in: &cancellables)
    }

    func setupSystemMessageObservers() {
        messagesManager.$currentMessage.sink { [weak self] message in
            guard let message
            else {
                Task { @MainActor in
                    self?.isSnackBarDisplayed = false
                }
                return
            }
            Task { @MainActor in
                self?.snackBarMessage = message
                withAnimation {
                    self?.isSnackBarDisplayed = true
                }
            }
        }
        .store(in: &cancellables)
    }

    func setupIsMnemonicImportedObserver() {
        appSettings.$isCredentialImportedPublisher
            .removeDuplicates()
            .receive(on: DispatchQueue.main)
            .sink { [weak self] _ in
                MainActor.assumeIsolated {
                    self?.updateConnectButtonStateIfMnemonicImported()
                }
            }
            .store(in: &cancellables)
    }
}

extension HomeViewModel {
    @MainActor func updateUI(with status: TunnelStatus) {
        guard status != lastTunnelStatus else { return }
        let newStatus: TunnelStatus
#if os(iOS)
        // TODO: remove once tunnel supports reconnect
        // Fake satus, until we get support from the tunnel
        if connectionManager.isReconnecting &&
            (status == .disconnecting || status == .disconnected || status == .connecting) {
            newStatus = .reasserting
        } else {
            newStatus = status
        }
        if newStatus == .connected {
            impactGenerator.success()
        }
#elseif os(macOS)
        newStatus = status
#endif
        lastTunnelStatus = newStatus
        withAnimation { [weak self] in
            guard let self else { return }
            statusButtonConfig = StatusButtonConfig(
                tunnelStatus: newStatus,
                hasInternet: networkMonitor.isAvailable,
                subscriptionDidExpire: isLastErrorSubscriptionExpired()
            )
            connectButtonState = ConnectButtonState(
                tunnelStatus: newStatus,
                isCredentialImported: credentialsManager.isValidCredentialImported
            )

            if let lastError {
                statusInfoState = .error(message: lastError.localizedDescription)
            } else {
                statusInfoState = StatusInfoState(
                    tunnelStatus: newStatus,
                    isOnline: networkMonitor.isAvailable,
                    retryAttempt: connectionManager.connectionRetryAttempt,
                    tunnelConnectingState: connectionManager.tunnelConnectingState,
                    subscriptionStatus: credentialsManager.accountSummary?.subscription?.status
                )
            }

            if newStatus == .connected {
                resetStatusInfoState()
            }

            displayEnableStatisticsSnackBarCTAIfNeeded()
        }
    }
}

extension HomeViewModel {
    func changeConnectionType(with type: ConnectionType) {
        impactGenerator.softImpact()
        guard connectionManager.connectionType != type else { return }
        connectionManager.connectionType = type
    }
}
