import Combine
import SwiftUI
import AppSettings
import ConnectionManager
import Constants
import ConfigurationManager
import CountriesManagerTypes
import CredentialsManager
import ExternalLinkManager
import GatewayManager
import MessagesManager
import MessageModels
import NetworkMonitor
import NymVPNRpc
import Settings
import TunnelMixnet
import TunnelStatus
import Tunnels
import UIComponents
import ImpactGenerator
#if os(macOS)
import GRPCManager
#endif

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

    @ObservedObject var connectionManager: ConnectionManager
    var cancellables = Set<AnyCancellable>()
    var tunnelStatusUpdateCancellable: AnyCancellable?
    var tunnelRetryAttemptCancellable: AnyCancellable?
    var tunnelConnectingStateCancellable: AnyCancellable?
    var lastTunnelStatus = TunnelStatus.disconnected
    var lastError: Error?

#if os(macOS)
    @Published public var isServing = false
#endif

    @MainActor @Published var activeTunnel: Tunnel?
    @MainActor @Published var statusButtonConfig = StatusButtonConfig.disconnected

    /// Use updateStatusInfoState(with:) to update the statusInfoState
    @MainActor @Published var statusInfoState = StatusInfoState.initialising

    /// Info from connecting/connected data, current gatewayId, that tunnel is connecting/connected to
    @MainActor @Published var connectionInfoData: ConnectionInfoData?

    @MainActor @Published var connectButtonState = ConnectButtonState.connect
    @MainActor @Published var isModeInfoOverlayDisplayed = false
    @MainActor @Published var isOfflineOverlayDisplayed = false
    @MainActor @Published var isUpdateAvailableOverlayDisplayed = false
    @MainActor @Published var isStatisticsOverlayDisplayed = false
    @MainActor @Published var snackBarMessage: SnackBarMessage?
    @MainActor @Published var isSnackBarDisplayed = false {
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

    @MainActor @Published public var splashScreenDidDisplay = false

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

    @MainActor func navigateToGatewayDetails(for hopType: HopType) {
        switch hopType {
        case .entry:
            if let entryGatewayId = connectionInfoData?.entryGatewayId ?? connectionManager.entryGateway.gatewayId,
               let entryGateway = gatewayManager.gateway(with: entryGatewayId, gatewayType: .entry) {
                navigateToGatewayDetails(gateway: entryGateway, hopType: hopType)
            }
        case .exit:
            if let exitGatewayId = connectionInfoData?.exitGatewayId ?? connectionManager.exitRouter.gatewayId,
               let exitGateway = gatewayManager.gateway(with: exitGatewayId, gatewayType: .exit) {
                navigateToGatewayDetails(gateway: exitGateway, hopType: hopType)
            }
        }
    }

    @MainActor private func navigateToGatewayDetails(gateway: GatewayNode, hopType: HopType) {
        path.append(HomeLink.gatewayDetails(gateway: gateway, hopType: hopType))
    }

    @MainActor func navigateToAddCredentials() {
        path.append(HomeLink.settings)
#if os(iOS)
        path.append(SettingLink.createAccountWelcome)
#elseif os(macOS)
        path.append(SettingLink.addCredentials)
#endif
    }

    @MainActor func navigateToPlanPurchase() {
#if os(iOS)
        path.append(HomeLink.settings)
        path.append(SettingLink.planPurchase(shouldDisplayBackButton: true))
#elseif os(macOS)
        try? externalLinkManager.openExternalURL(urlString: configurationManager.accountLinks?.account)
#endif
    }

#if os(macOS)
    func navigateToDaemonEnable() {
        path.append(HomeLink.settings)
        path.append(SettingLink.daemonEnable)
    }
#endif
}

// MARK: - Configuration -
private extension HomeViewModel {
    func setup() {
        setupTunnelManagerObservers()
        setupUpdateRequiredObserver()
        setupGatewayManagerObserver()
        setupSystemMessageObservers()

#if os(iOS)
        setupConnectionErrorObservers()
        setupNetworkMonitorObservers()
        setupIsMnemonicImportedObserver()
#elseif os(macOS)
        setupGRPCManagerObservers()
#endif
    }

    func setupTunnelManagerObservers() {
        connectionManager.$isTunnelManagerLoaded.sink { [weak self] result in
            switch result {
            case .success, .none:
                self?.resetStatusInfoState()
            case let .failure(error):
                let errorDescription = if let rpcError = error as? RpcError {
                    rpcError.message()
                } else {
                    error.localizedDescription
                }
                
                self?.updateStatusInfoState(with: .error(message: errorDescription))
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

    func setupGatewayManagerObserver() {
        gatewayManager.$lastError.sink { [weak self] error in
            self?.lastError = error
        }
        .store(in: &cancellables)
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
            connectButtonState = ConnectButtonState(tunnelStatus: newStatus)

            if let lastError {
                statusInfoState = .error(message: lastError.localizedDescription)
            } else {
                statusInfoState = StatusInfoState(
                    tunnelStatus: newStatus,
                    isOnline: networkMonitor.isAvailable,
                    retryAttempt: connectionManager.connectionRetryAttempt,
                    tunnelConnectingState: connectionManager.tunnelConnectingState
                )
            }

            if newStatus == .connected {
                resetStatusInfoState()
            }

            displayEnableStatisticsSnackBarCTAIfNeeded()
        }
    }

    func updateConnectButtonState(with newState: ConnectButtonState) {
        Task { @MainActor in
            guard newState != connectButtonState else { return }
            connectButtonState = newState
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
