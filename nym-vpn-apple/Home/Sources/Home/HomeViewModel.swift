import Combine
import SwiftUI
import AppSettings
import ConnectionManager
import CountriesManager
import TunnelMixnet
import Tunnels
import UIComponents

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
    var entryHopButtonViewModel = HopButtonViewModel(hopType: .entry)
    var exitHopButtonViewModel = HopButtonViewModel(hopType: .exit)
    @Published var selectedNetwork: NetworkButtonViewModel.ButtonType

    // If no time connected is shown, should be set to empty string,
    // so the time connected label would not disappear and re-center other UI elements.
    @Published var timeConnected = " "
    @Published var statusButtonConfig = StatusButtonConfig.disconnected
    @Published var statusInfoState = StatusInfoState.initialising
    @Published var connectButtonState = ConnectButtonState.connect
    public var screenSize: CGSize?

    public init(
        selectedNetwork: NetworkButtonViewModel.ButtonType,
        appSettings: AppSettings = AppSettings.shared,
        connectionManager: ConnectionManager = ConnectionManager.shared,
        countriesManager: CountriesManager = CountriesManager.shared
    ) {
        self.selectedNetwork = selectedNetwork
        self.appSettings = appSettings
        self.connectionManager = connectionManager
        self.countriesManager = countriesManager
        super.init()

        setup()
    }
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
}

// MARK: - Helpers -

public extension HomeViewModel {
    func isSmallScreen() -> Bool {
        guard let screenSize else { return false }
        return screenSize.width <= 375 && screenSize.height <= 647
    }

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
        guard let exitRouter = connectionManager.exitRouter
        else {
            // TODO: show error if no exit router
            return
        }
        var config = MixnetConfig(exitRouter: exitRouter)

        switch selectedNetwork {
        case .mixnet2hop:
            config = MixnetConfig(
                entryGateway: connectionManager.entryGateway,
                exitRouter: exitRouter,
                isTwoHopEnabled: true
            )
        case .mixnet5hop:
            config = MixnetConfig(
                entryGateway: connectionManager.entryGateway,
                exitRouter: exitRouter,
                isTwoHopEnabled: false
            )
        case .wireguard:
            break
        }

        connectionManager.connectDisconnect(with: config)
    }
}

// MARK: - Configuration -
private extension HomeViewModel {
    func setup() {
        setupDateFormatter()
        setupTunnelManagerObservers()
        setupAppSettingsObserver()
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

        connectionManager.$currentTunnel.sink { [weak self] tunnel in
            guard let tunnel else { return }
            self?.activeTunnel = tunnel
            self?.configureTunnelStatusObservation(with: tunnel)
        }
        .store(in: &cancellables)
    }

    func setupDateFormatter() {
        dateFormatter.allowedUnits = [.hour, .minute, .second]
        dateFormatter.zeroFormattingBehavior = .pad
    }

    func setupAppSettingsObserver() {
        appSettings.$isEntryLocationSelectionOnPublisher.sink { [weak self] _ in
            self?.fetchCountries()
        }
        .store(in: &cancellables)
    }

    func configureTunnelStatusObservation(with tunnel: Tunnel) {
        tunnel.$status.sink { [weak self] status in
            self?.statusButtonConfig = StatusButtonConfig(tunnelStatus: status)
            self?.statusInfoState = StatusInfoState(tunnelStatus: status)
            self?.connectButtonState = ConnectButtonState(tunnelStatus: status)
        }
        .store(in: &cancellables)
    }

    func fetchCountries() {
        do {
            try countriesManager.fetchCountries(shouldFetchEntryCountries: shouldShowEntryHop())
        } catch let error {
            print("🔥 ERROR: \(error.localizedDescription)")
            statusInfoState = .error(message: error.localizedDescription)
        }
    }
}
