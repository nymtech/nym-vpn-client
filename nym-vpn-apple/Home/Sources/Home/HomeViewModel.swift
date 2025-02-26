import Combine
import SwiftUI
import AppSettings
import ConnectionManager
import CountriesManager
import CredentialsManager
import ExternalLinkManager
import GatewayManager
import NetworkMonitor
import Settings
import SystemMessageManager
import TunnelMixnet
import TunnelStatus
import Tunnels
import UIComponents
#if os(iOS)
import ImpactGenerator
#elseif os(macOS)
import GRPCManager
import HelperInstall
import HelperManager
#endif

public class HomeViewModel: HomeFlowState {
    private var tunnelStatusUpdateCancellable: AnyCancellable?

    let title = "NymVPN".localizedString
    let connectToLocalizedTitle = "connectTo".localizedString
    let networkSelectLocalizedTitle = "selectNetwork".localizedString

    let appSettings: AppSettings
    let countriesManager: CountriesManager
    let credentialsManager: CredentialsManager
    let externalLinkManager: ExternalLinkManager
    let gatewayManager: GatewayManager
    let networkMonitor: NetworkMonitor
#if os(iOS)
    let impactGenerator: ImpactGenerator
#elseif os(macOS)
    let grpcManager: GRPCManager
    let helperManager: HelperManager
#endif
    let systemMessageManager: SystemMessageManager
    let anonymousButtonViewModel = NetworkButtonViewModel(type: .mixnet5hop)
    let fastButtonViewModel = NetworkButtonViewModel(type: .wireguard)

    var cancellables = Set<AnyCancellable>()
    var connectionManager: ConnectionManager
    var lastTunnelStatus = TunnelStatus.disconnected
    var lastError: Error?

    // If no time connected is shown, should be set to empty string,
    // so the time connected label would not disappear and re-center other UI elements.
    @Published var timeConnected: Date?
    @MainActor @Published var activeTunnel: Tunnel?
    @MainActor @Published var statusButtonConfig = StatusButtonConfig.disconnected
    @MainActor @Published var statusInfoState = StatusInfoState.initialising
    @MainActor @Published var connectButtonState = ConnectButtonState.connect
    @MainActor @Published var isModeInfoOverlayDisplayed = false
    @MainActor @Published var isOfflineOverlayDisplayed = false
    @MainActor @Published var snackBarMessage = ""
    @MainActor @Published var isSnackBarDisplayed = false {
        didSet {
            Task(priority: .background) {
                try? await Task.sleep(for: .seconds(1))
                guard !isSnackBarDisplayed else { return }
                systemMessageManager.messageDidClose()
            }
        }
    }

    var offlineOverlayConfiguration: ActionDialogConfiguration {
        ActionDialogConfiguration(
            iconImageName: "exclamationmark.circle",
            titleLocalizedString: "home.modal.noInternetConnection.title".localizedString,
            subtitleLocalizedString: "home.modal.noInternetConnection.subtitle".localizedString,
            yesLocalizedString: "close".localizedString
        )
    }

    @MainActor @Published public var splashScreenDidDisplay = false

#if os(iOS)
    public init(
        appSettings: AppSettings = .shared,
        connectionManager: ConnectionManager = .shared,
        countriesManager: CountriesManager = .shared,
        credentialsManager: CredentialsManager = .shared,
        networkMonitor: NetworkMonitor = .shared,
        externalLinkManager: ExternalLinkManager = .shared,
        gatewayManager: GatewayManager = .shared,
        impactGenerator: ImpactGenerator = .shared,
        systemMessageManager: SystemMessageManager = .shared
    ) {
        self.appSettings = appSettings
        self.connectionManager = connectionManager
        self.countriesManager = countriesManager
        self.credentialsManager = credentialsManager
        self.externalLinkManager = externalLinkManager
        self.gatewayManager = gatewayManager
        self.impactGenerator = impactGenerator
        self.networkMonitor = networkMonitor
        self.systemMessageManager = systemMessageManager
        super.init()

        setup()
    }
#elseif os(macOS)
    public init(
        appSettings: AppSettings = .shared,
        connectionManager: ConnectionManager = .shared,
        countriesManager: CountriesManager = .shared,
        credentialsManager: CredentialsManager = .shared,
        networkMonitor: NetworkMonitor = .shared,
        grpcManager: GRPCManager = .shared,
        helperManager: HelperManager = .shared,
        externalLinkManager: ExternalLinkManager = .shared,
        gatewayManager: GatewayManager = .shared,
        systemMessageManager: SystemMessageManager = .shared
    ) {
        self.appSettings = appSettings
        self.connectionManager = connectionManager
        self.countriesManager = countriesManager
        self.credentialsManager = credentialsManager
        self.networkMonitor = networkMonitor
        self.grpcManager = grpcManager
        self.helperManager = helperManager
        self.externalLinkManager = externalLinkManager
        self.gatewayManager = gatewayManager
        self.systemMessageManager = systemMessageManager
        super.init()

        setup()
    }
#endif

    deinit {
        cancellables.forEach { $0.cancel() }
    }
}

// MARK: - Navigation -

public extension HomeViewModel {
    @MainActor func navigateToSettings() {
        path.append(HomeLink.settings)
    }

    @MainActor func navigateToEntryGateways() {
        path.append(HomeLink.entryGateways)
    }

    @MainActor func navigateToExitGateways() {
        path.append(HomeLink.exitGateways)
    }

    @MainActor func navigateToAddCredentials() {
        path.append(HomeLink.settings)
        path.append(SettingLink.addCredentials)
    }

#if os(macOS)
    @MainActor func navigateToInstallHelper() {
        let action = HelperAfterInstallAction { [weak self] in
            self?.connectDisconnect()
        }
        path.append(HomeLink.installHelper(afterInstallAction: action))
    }
#endif
}

// MARK: - Configuration -
private extension HomeViewModel {
    func setup() {
        setupTunnelManagerObservers()
        setupConnectionErrorObservers()

#if os(macOS)
        setupGRPCManagerObservers()
#endif
        setupCountriesManagerObservers()
        setupGatewayManagerObserver()
        setupSystemMessageObservers()
        setupNetworkMonitorObservers()
        updateTimeConnected()
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
        connectionManager.$activeTunnel.sink { [weak self] tunnel in
            guard let tunnel, let self else { return }
            Task { @MainActor in
                self.activeTunnel = tunnel
            }
            self.configureTunnelStatusObservation(with: tunnel)
            self.updateTimeConnected()
        }
        .store(in: &cancellables)
#endif
    }

    func setupCountriesManagerObservers() {
        countriesManager.$lastError.sink { [weak self] error in
            self?.lastError = error
        }
        .store(in: &cancellables)
    }

    func setupGatewayManagerObserver() {
        gatewayManager.$lastError.sink { [weak self] error in
            self?.lastError = error
        }
        .store(in: &cancellables)
    }

    func setupSystemMessageObservers() {
        systemMessageManager.$currentMessage.sink { [weak self] message in
            guard !message.isEmpty
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

    func setupNetworkMonitorObservers() {
        // We use networkMonitor only as a source of truth for iOS disconnected state.
        // For macOS - we rely on daemon tunnel states.
#if os(iOS)
        networkMonitor.$isAvailable
            .removeDuplicates()
            .debounce(for: .seconds(0.3), scheduler: DispatchQueue.global(qos: .background))
            .sink { [weak self] isAvailable in
                self?.offlineState(with: isAvailable)
            }
            .store(in: &cancellables)
#endif
    }

    func setupConnectionErrorObservers() {
#if os(iOS)
        connectionManager.$lastError
            .receive(on: DispatchQueue.main)
            .sink { error in
                MainActor.assumeIsolated {
                    self.updateLastError(error)
                }
            }
            .store(in: &cancellables)
#endif
    }
#if os(iOS)
    func configureTunnelStatusObservation(with tunnel: Tunnel) {
        tunnelStatusUpdateCancellable = tunnel.$status
            .removeDuplicates()
            .receive(on: DispatchQueue.main)
            .sink { [weak self] status in
                self?.updateUI(with: status)
                self?.updateTimeConnected()
            }
    }
#endif

    func offlineState(with hasInternet: Bool) {
        Task { @MainActor [weak self] in
            guard let self else { return }
            withAnimation { [weak self] in
                guard let self else { return }
                statusButtonConfig = StatusButtonConfig(tunnelStatus: lastTunnelStatus, hasInternet: hasInternet)
                statusInfoState = StatusInfoState(hasInternet: hasInternet)
            }
        }
    }

    func fetchCountries() {
        countriesManager.fetchCountries()
    }
}

extension HomeViewModel {
    func updateUI(with status: TunnelStatus) {
        guard status != lastTunnelStatus else { return }
        let newStatus: TunnelStatus
        // Fake satus, until we get support from the tunnel
        if connectionManager.isReconnecting &&
            (status == .disconnecting || status == .disconnected || status == .connecting) {
            newStatus = .reasserting
        } else if connectionManager.isDisconnecting &&
                    (status == .connecting || status == .connected) {
            newStatus = .disconnecting
        } else {
            newStatus = status
        }
#if os(iOS)
        if status == .connected && !connectionManager.isDisconnecting {
            impactGenerator.success()
        }
#endif
        lastTunnelStatus = newStatus
        Task { @MainActor [weak self] in
            guard let self else { return }
            withAnimation { [weak self] in
                guard let self else { return }
                statusButtonConfig = StatusButtonConfig(
                    tunnelStatus: newStatus,
                    hasInternet: networkMonitor.isAvailable
                )
                connectButtonState = ConnectButtonState(tunnelStatus: newStatus)

                if let lastError {
                    statusInfoState = .error(message: lastError.localizedDescription)
                } else {
                    statusInfoState = StatusInfoState(tunnelStatus: newStatus, isOnline: networkMonitor.isAvailable)
                }
            }
        }
    }

    func updateConnectButtonState(with newState: ConnectButtonState) {
        Task { @MainActor in
            guard newState != connectButtonState else { return }
            connectButtonState = newState
        }
    }
}
