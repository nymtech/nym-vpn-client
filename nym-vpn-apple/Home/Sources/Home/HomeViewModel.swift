import Combine
import SwiftUI
import AppSettings
import ConnectionManager
import CountriesManager
import CredentialsManager
import ExternalLinkManager
import Settings
import SystemMessageManager
import TunnelMixnet
import TunnelStatus
import Tunnels
import UIComponents
#if os(iOS)
import ImpactGenerator
#endif
#if os(macOS)
import GRPCManager
import HelperInstallManager
#endif

public class HomeViewModel: HomeFlowState {
    private var cancellables = Set<AnyCancellable>()
    private var tunnelStatusUpdateCancellable: AnyCancellable?
    private var lastError: Error?
    @MainActor @Published private var activeTunnel: Tunnel?

    let title = "NymVPN".localizedString
    let connectToLocalizedTitle = "connectTo".localizedString
    let networkSelectLocalizedTitle = "selectNetwork".localizedString

    let appSettings: AppSettings
    let countriesManager: CountriesManager
    let credentialsManager: CredentialsManager
    let externalLinkManager: ExternalLinkManager
#if os(iOS)
    let impactGenerator: ImpactGenerator
#endif
#if os(macOS)
    let grpcManager: GRPCManager
    let helperInstallManager: HelperInstallManager
#endif
    let systemMessageManager: SystemMessageManager
    let anonymousButtonViewModel = NetworkButtonViewModel(type: .mixnet5hop)
    let fastButtonViewModel = NetworkButtonViewModel(type: .wireguard)

    var connectionManager: ConnectionManager
    var lastTunnelStatus = TunnelStatus.disconnected

    // If no time connected is shown, should be set to empty string,
    // so the time connected label would not disappear and re-center other UI elements.
    @Published var timeConnected: Date?
    @MainActor @Published var statusButtonConfig = StatusButtonConfig.disconnected
    @MainActor @Published var statusInfoState = StatusInfoState.initialising
    @MainActor @Published var connectButtonState = ConnectButtonState.connect
    @MainActor @Published var isModeInfoOverlayDisplayed = false
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

    @MainActor @Published public var splashScreenDidDisplay = false

#if os(iOS)
    public init(
        appSettings: AppSettings = .shared,
        connectionManager: ConnectionManager = .shared,
        countriesManager: CountriesManager = .shared,
        credentialsManager: CredentialsManager = .shared,
        externalLinkManager: ExternalLinkManager = .shared,
        impactGenerator: ImpactGenerator = .shared,
        systemMessageManager: SystemMessageManager = .shared
    ) {
        self.appSettings = appSettings
        self.connectionManager = connectionManager
        self.countriesManager = countriesManager
        self.credentialsManager = credentialsManager
        self.externalLinkManager = externalLinkManager
        self.impactGenerator = impactGenerator
        self.systemMessageManager = systemMessageManager
        super.init()

        setup()
    }
#endif

#if os(macOS)
    public init(
        appSettings: AppSettings = .shared,
        connectionManager: ConnectionManager = .shared,
        countriesManager: CountriesManager = .shared,
        credentialsManager: CredentialsManager = .shared,
        grpcManager: GRPCManager = .shared,
        helperInstallManager: HelperInstallManager = .shared,
        externalLinkManager: ExternalLinkManager = .shared,
        systemMessageManager: SystemMessageManager = .shared
    ) {
        self.appSettings = appSettings
        self.connectionManager = connectionManager
        self.countriesManager = countriesManager
        self.credentialsManager = credentialsManager
        self.grpcManager = grpcManager
        self.helperInstallManager = helperInstallManager
        self.externalLinkManager = externalLinkManager
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
    func navigateToSettings() {
        Task { @MainActor in
            path.append(HomeLink.settings)
        }
    }

    func navigateToFirstHopSelection() {
        Task { @MainActor in
            path.append(HomeLink.entryHop)
        }
    }

    func navigateToLastHopSelection() {
        Task { @MainActor in
            path.append(HomeLink.exitHop)
        }
    }

    @MainActor func navigateToAddCredentials() {
        path.append(HomeLink.settings)
        path.append(SettingLink.addCredentials)
    }
}

// MARK: - Connection -
public extension HomeViewModel {
    func connectDisconnect() {
        guard connectionManager.currentTunnelStatus != .disconnecting
        else {
            return
        }
#if os(iOS)
        impactGenerator.impact()
#endif
        Task {
            lastError = nil
            resetStatusInfoState()
#if os(macOS)
            guard helperInstallManager.daemonState != .installing else { return }
            do {
                try await helperInstallManager.installIfNeeded()
            } catch {
                updateStatusInfoState(with: .error(message: error.localizedDescription))
                updateConnectButtonState(with: .connect)
                return
            }
#endif
            guard credentialsManager.isValidCredentialImported
            else {
                await navigateToAddCredentials()
                return
            }

            do {
                try await connectionManager.connectDisconnect()
            } catch let error {
                updateStatusInfoState(with: .error(message: error.localizedDescription))
#if os(iOS)
                impactGenerator.error()
#endif
            }
        }
    }
}

// MARK: - Configuration -
private extension HomeViewModel {
    func setup() {
        setupTunnelManagerObservers()
        setupConnectionErrorObservers()

#if os(macOS)
        setupGRPCManagerObservers()
        setupDaemonStateObserver()
#endif
        setupCountriesManagerObservers()
        setupSystemMessageObservers()
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

    func setupConnectionErrorObservers() {
#if os(iOS)
        connectionManager.$lastError.sink { [weak self] error in
            self?.lastError = error
            if let error {
                self?.updateStatusInfoState(with: .error(message: error.localizedDescription))
            }
        }
        .store(in: &cancellables)
#endif

#if os(macOS)
        grpcManager.$lastError.sink { [weak self] error in
            self?.lastError = error
        }
        .store(in: &cancellables)
#endif
    }
#if os(iOS)
    func configureTunnelStatusObservation(with tunnel: Tunnel) {
        tunnelStatusUpdateCancellable = tunnel.$status
            .debounce(for: .seconds(0.3), scheduler: DispatchQueue.global(qos: .background))
            .removeDuplicates()
            .receive(on: RunLoop.main)
            .sink { [weak self] status in
                self?.updateUI(with: status)
                self?.updateTimeConnected()
            }
    }
#endif

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
            guard let activeTunnel,
                  activeTunnel.status == .connected,
                  let connectedDate = activeTunnel.tunnel.connection.connectedDate
            else {
                timeConnected = nil
                return
            }
            timeConnected = connectedDate
        }
    }
}
#endif

// MARK: - macOS -
#if os(macOS)
private extension HomeViewModel {
    func setupGRPCManagerObservers() {
        grpcManager.$tunnelStatus
            .debounce(for: .seconds(0.15), scheduler: DispatchQueue.global(qos: .background))
            .receive(on: RunLoop.main)
            .sink { [weak self] status in
                self?.updateUI(with: status)
                self?.updateTimeConnected()
            }
            .store(in: &cancellables)
    }

    func setupDaemonStateObserver() {
        helperInstallManager.$daemonState.sink { [weak self] state in
            switch state {
            case .installing:
                self?.updateStatusInfoState(with: .installingDaemon)
                self?.updateConnectButtonState(with: .installingDaemon)
            case .installed:
                self?.updateStatusInfoState(with: .unknown)
                self?.updateConnectButtonState(with: .connect)
            case .unknown, .running:
                break
            }
        }
        .store(in: &cancellables)
    }

    func updateConnectedStartDateMacOS(with status: TunnelStatus) {
        guard status == .connected, !connectionManager.isDisconnecting else { return }
        grpcManager.status()
    }

    func updateTimeConnected() {
        Task { @MainActor [weak self] in
            guard let self,
                  grpcManager.tunnelStatus == .connected,
                  let connectedDate = grpcManager.connectedDate
            else {
                self?.timeConnected = nil
                return
            }
            self.timeConnected = connectedDate
        }
    }
}
#endif

private extension HomeViewModel {
    func resetStatusInfoState() {
        updateStatusInfoState(with: .unknown)
    }

    func updateStatusInfoState(with newState: StatusInfoState) {
        Task { @MainActor in
            guard newState != statusInfoState else { return }
            statusInfoState = newState
        }
    }

    func updateConnectButtonState(with newState: ConnectButtonState) {
        Task { @MainActor in
            guard newState != connectButtonState else { return }
            connectButtonState = newState
        }
    }
}
