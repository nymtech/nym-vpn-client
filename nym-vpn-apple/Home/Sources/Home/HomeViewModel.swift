import Combine
import SwiftUI
import AppSettings
import ConnectionManager
import CountriesManager
import CredentialsManager
import Settings
import TunnelMixnet
import TunnelStatus
import Tunnels
import UIComponents

#if os(macOS)
import GRPCManager
import HelperManager
#endif

public class HomeViewModel: HomeFlowState {
    private let dateFormatter = DateComponentsFormatter()

    private var timer = Timer()
    private var cancellables = Set<AnyCancellable>()
    @Published private var activeTunnel: Tunnel?

    let title = "NymVPN".localizedString
    let connectToLocalizedTitle = "connectTo".localizedString
    let networkSelectLocalizedTitle = "selectNetwork".localizedString

    var appSettings: AppSettings
    var connectionManager: ConnectionManager
    var countriesManager: CountriesManager
    var credentialsManager: CredentialsManager
#if os(macOS)
    var grpcManager: GRPCManager
    var helperManager: HelperManager
#endif
    var entryHopButtonViewModel = HopButtonViewModel(hopType: .entry)
    var exitHopButtonViewModel = HopButtonViewModel(hopType: .exit)
    @Published var selectedNetwork: NetworkButtonViewModel.ButtonType

    // If no time connected is shown, should be set to empty string,
    // so the time connected label would not disappear and re-center other UI elements.
    @Published var timeConnected = " "
    @Published var statusButtonConfig = StatusButtonConfig.disconnected
    @Published var statusInfoState = StatusInfoState.initialising
    @Published var connectButtonState = ConnectButtonState.connect
    @Published var shouldShowHopSection = false

#if os(iOS)
    public init(
        selectedNetwork: NetworkButtonViewModel.ButtonType,
        appSettings: AppSettings = AppSettings.shared,
        connectionManager: ConnectionManager = ConnectionManager.shared,
        countriesManager: CountriesManager = CountriesManager.shared,
        credentialsManager: CredentialsManager = CredentialsManager.shared
    ) {
        self.selectedNetwork = selectedNetwork
        self.appSettings = appSettings
        self.connectionManager = connectionManager
        self.countriesManager = countriesManager
        self.credentialsManager = credentialsManager
        super.init()

        setup()
    }
#endif
#if os(macOS)
    public init(
        selectedNetwork: NetworkButtonViewModel.ButtonType,
        appSettings: AppSettings = AppSettings.shared,
        connectionManager: ConnectionManager = ConnectionManager.shared,
        countriesManager: CountriesManager = CountriesManager.shared,
        credentialsManager: CredentialsManager = CredentialsManager.shared,
        grpcManager: GRPCManager = GRPCManager.shared,
        helperManager: HelperManager = HelperManager.shared
    ) {
        self.selectedNetwork = selectedNetwork
        self.appSettings = appSettings
        self.connectionManager = connectionManager
        self.countriesManager = countriesManager
        self.credentialsManager = credentialsManager
        self.grpcManager = grpcManager
        self.helperManager = helperManager
        super.init()

        setup()
    }
#endif
}

// MARK: - Navigation -

public extension HomeViewModel {
    func navigateToSettings() {
        path.append(HomeLink.settings)
    }

    func navigateToFirstHopSelection() {
        path.append(HomeLink.entryHop)
    }

    func navigateToLastHopSelection() {
        path.append(HomeLink.exitHop)
    }

    func navigateToAddCredentials() {
        path.append(HomeLink.settings)
        path.append(SettingsLink.addCredentials)
    }
}

// MARK: - Helpers -

public extension HomeViewModel {
    func shouldShowEntryHop() -> Bool {
        appSettings.isEntryLocationSelectionOn && !(countriesManager.entryCountries?.isEmpty ?? false)
    }

    func updateTimeConnected() {
        let emptyTime = " "
        guard
            let activeTunnel,
            activeTunnel.status == .connected,
            let connectedDate = activeTunnel.tunnel.connection.connectedDate
        else {

            guard timeConnected != emptyTime else { return }
            timeConnected = emptyTime
            return
        }
        timeConnected = dateFormatter.string(from: connectedDate, to: Date()) ?? emptyTime
    }

    func configureConnectedTimeTimer() {
        timer = Timer.scheduledTimer(withTimeInterval: 0.3, repeats: true) { [weak self] _ in
            self?.updateTimeConnected()
        }
    }

    func stopConnectedTimeTimerUpdates() {
        timer.invalidate()
    }
}

// MARK: - Connection -
public extension HomeViewModel {
    func connectDisconnect() {
        installHelperIfNeeded()

        guard appSettings.isCredentialImported
        else {
            navigateToAddCredentials()
            return
        }

        guard let exitRouter = connectionManager.exitRouter
        else {
            statusInfoState = .error(message: "TODO")
            return
        }

        do {
            let credentialURL = try credentialsManager.dataFolderURL()
            var config = MixnetConfig(exitRouter: exitRouter, credentialsDataPath: credentialURL.path())

            switch selectedNetwork {
            case .mixnet2hop:
                config = MixnetConfig(
                    entryGateway: connectionManager.entryGateway,
                    exitRouter: exitRouter,
                    isTwoHopEnabled: true,
                    credentialsDataPath: credentialURL.path()
                )
            case .mixnet5hop:
                config = MixnetConfig(
                    entryGateway: connectionManager.entryGateway,
                    exitRouter: exitRouter,
                    isTwoHopEnabled: false,
                    credentialsDataPath: credentialURL.path()
                )
            case .wireguard:
                break
            }

            connectionManager.connectDisconnect(with: config)
        } catch let error {
            statusInfoState = .error(message: error.localizedDescription)
        }
    }
}

// MARK: - Configuration -
private extension HomeViewModel {
    func setup() {
        setupDateFormatter()
        setupTunnelManagerObservers()
        setupAppSettingsObservers()
        setupCountriesManagerObservers()
        fetchCountries()
    }

    func setupTunnelManagerObservers() {
        connectionManager.$isTunnelManagerLoaded.sink { [weak self] result in
            switch result {
            case .success, .none:
                self?.statusInfoState = .unknown
            case let .failure(error):
                self?.statusInfoState = .error(message: error.localizedDescription)
            }
        }
        .store(in: &cancellables)
#if os(iOS)
        connectionManager.$currentTunnel.sink { [weak self] tunnel in
            guard let tunnel else { return }
            self?.activeTunnel = tunnel
            self?.configureTunnelStatusObservation(with: tunnel)
        }
        .store(in: &cancellables)
#endif
#if os(macOS)
        grpcManager.$tunnelStatus.sink { [weak self] status in
            self?.updateUI(with: status)
        }
        .store(in: &cancellables)
#endif
    }

    func setupDateFormatter() {
        dateFormatter.allowedUnits = [.hour, .minute, .second]
        dateFormatter.zeroFormattingBehavior = .pad
    }

    func setupAppSettingsObservers() {
        appSettings.$isEntryLocationSelectionOnPublisher.sink { [weak self] _ in
            self?.fetchCountries()
        }
        .store(in: &cancellables)
    }

    func setupCountriesManagerObservers() {
        countriesManager.$hasCountries.sink { [weak self] value in
            guard let self else { return }

            Task { @MainActor in
                self.shouldShowHopSection = value
            }
        }
        .store(in: &cancellables)

        countriesManager.$lastError.sink { [weak self] error in
            guard
                let self,
                let localizedDescription = error?.localizedDescription
            else {
                return
            }

            Task { @MainActor in
                self.statusInfoState = .error(message: localizedDescription)
            }
        }
        .store(in: &cancellables)
    }

#if os(iOS)
    func configureTunnelStatusObservation(with tunnel: Tunnel) {
        tunnel.$status.sink { [weak self] status in
            self?.updateUI(with: status)
        }
        .store(in: &cancellables)
    }
#endif

    func installHelperIfNeeded() {
#if os(macOS)
        // TODO: check if possible to split is helper running vs isHelperAuthorized
        guard helperManager.isHelperRunning() && helperManager.isHelperAuthorized()
        else {
            do {
                _ = try helperManager.authorizeAndInstallHelper()
            } catch let error {
                statusInfoState = .error(message: error.localizedDescription)
            }
            return
        }
#endif
    }

    func updateUI(with status: TunnelStatus) {
        Task { @MainActor in
            statusButtonConfig = StatusButtonConfig(tunnelStatus: status)
            statusInfoState = StatusInfoState(tunnelStatus: status)
            connectButtonState = ConnectButtonState(tunnelStatus: status)
        }
    }

    func fetchCountries() {
        do {
            try countriesManager.fetchCountries()
        } catch let error {
            statusInfoState = .error(message: error.localizedDescription)
        }
    }
}
