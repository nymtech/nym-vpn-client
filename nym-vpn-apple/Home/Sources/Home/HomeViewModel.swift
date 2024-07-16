import Combine
import SwiftUI
import AppSettings
import ConnectionManager
import CountriesManager
import ExternalLinkManager
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
    private var tunnelStatusUpdateCancellable: AnyCancellable?
    private var lastError: Error?
    @MainActor @Published private var activeTunnel: Tunnel?

    let title = "NymVPN".localizedString
    let connectToLocalizedTitle = "connectTo".localizedString
    let networkSelectLocalizedTitle = "selectNetwork".localizedString

    let appSettings: AppSettings
    let connectionManager: ConnectionManager
    let countriesManager: CountriesManager
    let externalLinkManager: ExternalLinkManager

#if os(macOS)
    let grpcManager: GRPCManager
    let helperManager: HelperManager
#endif
    let entryHopButtonViewModel = HopButtonViewModel(hopType: .entry)
    let exitHopButtonViewModel = HopButtonViewModel(hopType: .exit)
    let anonymousButtonViewModel = NetworkButtonViewModel(type: .mixnet5hop)
    let fastButtonViewModel = NetworkButtonViewModel(type: .mixnet2hop)

    // If no time connected is shown, should be set to empty string,
    // so the time connected label would not disappear and re-center other UI elements.
    @Published var timeConnected = " "
    @MainActor @Published var statusButtonConfig = StatusButtonConfig.disconnected
    @MainActor @Published var statusInfoState = StatusInfoState.initialising
    @MainActor @Published var connectButtonState = ConnectButtonState.connect
    @MainActor @Published var isModeInfoOverlayDisplayed = false

#if os(iOS)
    public init(
        appSettings: AppSettings = AppSettings.shared,
        connectionManager: ConnectionManager = ConnectionManager.shared,
        countriesManager: CountriesManager = CountriesManager.shared,
        externalLinkManager: ExternalLinkManager = ExternalLinkManager.shared
    ) {
        self.appSettings = appSettings
        self.connectionManager = connectionManager
        self.countriesManager = countriesManager
        self.externalLinkManager = externalLinkManager

        super.init()

        setup()
    }
#endif
#if os(macOS)
    public init(
        appSettings: AppSettings = AppSettings.shared,
        connectionManager: ConnectionManager = ConnectionManager.shared,
        countriesManager: CountriesManager = CountriesManager.shared,
        grpcManager: GRPCManager = GRPCManager.shared,
        helperManager: HelperManager = HelperManager.shared,
        externalLinkManager: ExternalLinkManager = ExternalLinkManager.shared
    ) {
        self.appSettings = appSettings
        self.connectionManager = connectionManager
        self.countriesManager = countriesManager
        self.grpcManager = grpcManager
        self.helperManager = helperManager
        self.externalLinkManager = externalLinkManager
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
        appSettings.isEntryLocationSelectionOn && !countriesManager.entryCountries.isEmpty
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
        Task {
            lastError = nil
            resetStatusInfoState()

#if os(macOS)
            guard await isHelperInstalled() else { return }
#endif

            guard appSettings.isCredentialImported
            else {
                navigateToAddCredentials()
                return
            }

            resetStatusInfoState()

            Task { @MainActor in
                do {
                    try connectionManager.connectDisconnect()
                } catch let error {
                    statusInfoState = .error(message: error.localizedDescription)
                }
            }
        }
    }
}

private extension  HomeViewModel {
#if os(macOS)
    func isHelperInstalled() async -> Bool {
        let isHelperInstalledAndRunning = await installHelperIfNeeded()

        guard isHelperInstalledAndRunning
        else {
            updateStatusInfoState(with: .error(message: "home.installDaemonFailure".localizedString))
            return false
        }

        guard helperManager.isHelperAuthorizedAndRunning()
        else {
            return false
        }
        return true
    }
#endif
}

// MARK: - Configuration -
private extension HomeViewModel {
    func setup() {
        setupDateFormatter()
        setupTunnelManagerObservers()
#if os(macOS)
        setupGRPCManagerObservers()
#endif
        setupCountriesManagerObservers()
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
        }
        .store(in: &cancellables)
#endif
    }

    func setupDateFormatter() {
        dateFormatter.allowedUnits = [.hour, .minute, .second]
        dateFormatter.zeroFormattingBehavior = .pad
    }

    func setupCountriesManagerObservers() {
        countriesManager.$lastError.sink { [weak self] error in
            self?.lastError = error
        }
        .store(in: &cancellables)
    }

#if os(iOS)
    func configureTunnelStatusObservation(with tunnel: Tunnel) {
        tunnelStatusUpdateCancellable = tunnel.$status
            .debounce(for: .seconds(0.3), scheduler: DispatchQueue.global(qos: .background))
            .removeDuplicates()
            .receive(on: RunLoop.main)
            .sink { [weak self] status in
                self?.updateUI(with: status)
            }
    }
#endif

    func updateUI(with status: TunnelStatus) {
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

        Task { @MainActor in
            statusButtonConfig = StatusButtonConfig(tunnelStatus: newStatus)
            connectButtonState = ConnectButtonState(tunnelStatus: newStatus)

            if let lastError {
                statusInfoState = .error(message: lastError.localizedDescription)
            } else {
                statusInfoState = StatusInfoState(tunnelStatus: newStatus)
            }
#if os(macOS)
            updateConnectedStartDateMacOS(with: status)
#endif
        }
    }

    func fetchCountries() {
        countriesManager.fetchCountries()
    }
}

// MARK: - iOS -
#if os(iOS)
private extension HomeViewModel {
    func updateTimeConnected() {
        Task { @MainActor in
            let emptyTimeText = " "
            guard let activeTunnel,
                  activeTunnel.status == .connected,
                  let connectedDate = activeTunnel.tunnel.connection.connectedDate
            else {
                guard timeConnected != emptyTimeText else { return }
                timeConnected = emptyTimeText
                return
            }
            timeConnected = dateFormatter.string(from: connectedDate, to: Date()) ?? emptyTimeText
        }
    }
}
#endif

// MARK: - macOS -
#if os(macOS)
private extension HomeViewModel {
    func setupGRPCManagerObservers() {
        grpcManager.$lastError.sink { [weak self] error in
            self?.lastError = error
        }
        .store(in: &cancellables)

        grpcManager.$tunnelStatus
            .debounce(for: .seconds(0.15), scheduler: DispatchQueue.global(qos: .background))
            .receive(on: RunLoop.main)
            .sink { [weak self] status in
                self?.updateUI(with: status)
            }
            .store(in: &cancellables)
    }

    func installHelperIfNeeded() async -> Bool {
        var isInstalledAndRunning = helperManager.isHelperAuthorizedAndRunning()
        // TODO: check if possible to split is helper running vs isHelperAuthorized
        guard isInstalledAndRunning
        else {
            do {
                updateStatusInfoState(with: .error(message: "home.installDaemon".localizedString))
                isInstalledAndRunning = try helperManager.authorizeAndInstallHelper()
                resetStatusInfoState()
            } catch let error {
                updateStatusInfoState(with: .error(message: error.localizedDescription))
            }
            return isInstalledAndRunning
        }
        return isInstalledAndRunning
    }

    func updateConnectedStartDateMacOS(with status: TunnelStatus) {
        guard status == .connected, !connectionManager.isDisconnecting else { return }
        grpcManager.status()
    }

    func updateTimeConnected() {
        Task { @MainActor in
            let emptyTimeText = " "
            guard grpcManager.tunnelStatus == .connected,
                  let connectedDate = grpcManager.connectedDate
            else {
                guard timeConnected != emptyTimeText else { return }
                timeConnected = emptyTimeText
                return
            }
            timeConnected = dateFormatter.string(from: connectedDate, to: Date()) ?? emptyTimeText
        }
    }
}
#endif

private extension HomeViewModel {
    func resetStatusInfoState() {
        Task { @MainActor in
            statusInfoState = .unknown
        }
    }

    func updateStatusInfoState(with newState: StatusInfoState) {
        Task { @MainActor in
            statusInfoState = newState
        }
    }
}
