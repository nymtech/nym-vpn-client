import Combine
import SwiftUI
import AppSettings
import ConnectionManager
import Tunnels
import UIComponents

public class HomeViewModel: HomeFlowState {
    private let appSettings: AppSettings
    private let screenSize: CGSize
    private let dateFormatter = DateComponentsFormatter()

    private var timer = Timer()
    private var cancellables = Set<AnyCancellable>()
    @ObservedObject private var connectionManager: ConnectionManager
    @Published private var activeTunnel: Tunnel?

    @Published var selectedNetwork: NetworkButtonViewModel.ButtonType

    // If no time connected is shown, should be set to empty string,
    // so the time connected label would not disappear and re-center other UI elements.
    @Published var timeConnected = " "
    @Published var statusButtonConfig = StatusButtonConfig.disconnected
    @Published var statusInfoState = StatusInfoState.initialising
    @Published var connectButtonState = ConnectButtonState.connect

    public init(
        screenSize: CGSize,
        selectedNetwork: NetworkButtonViewModel.ButtonType,
        appSettings: AppSettings = AppSettings.shared,
        connectionManager: ConnectionManager = ConnectionManager.shared
    ) {
        self.screenSize = screenSize
        self.selectedNetwork = selectedNetwork
        self.appSettings = appSettings
        self.connectionManager = connectionManager
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
        path.append(HomeLink.firstHop(text: ""))
    }

    func navigateToLastHopSelection() {
        path.append(HomeLink.lastHop)
    }
}

// MARK: - Helpers -

public extension HomeViewModel {
    func isSmallScreen() -> Bool {
        screenSize.width <= 375 && screenSize.height <= 647
    }

    func shouldShowEntryHop() -> Bool {
        appSettings.entryLocationSelectionIsOn
    }

    func updateTimeConnected() {
        guard
            let activeTunnel,
            activeTunnel.status == .connected,
            let connectedDate = activeTunnel.tunnel.connection.connectedDate
        else {
            timeConnected = " "
            return
        }
        timeConnected = dateFormatter.string(from: connectedDate, to: Date()) ?? ""
    }
}

// MARK: - Connection -

public extension HomeViewModel {
    func connectDisconnect() {
        connectionManager.connectDisconnect()
    }
}

private extension HomeViewModel {
    func setup() {
        setupDateFormatter()
        setupConnectedTimeTimer()
        setupTunnelManagerObservers()
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

    func setupConnectedTimeTimer() {
        timer = Timer.scheduledTimer(withTimeInterval: 0.3, repeats: true) { [weak self] _ in
            self?.updateTimeConnected()
        }
    }

    func configureTunnelStatusObservation(with tunnel: Tunnel) {
        tunnel.$status.sink { [weak self] status in
            self?.statusButtonConfig = StatusButtonConfig(tunnelStatus: status)
            self?.statusInfoState = StatusInfoState(tunnelStatus: status)
            self?.connectButtonState = ConnectButtonState(tunnelStatus: status)
        }
        .store(in: &cancellables)
    }
}
