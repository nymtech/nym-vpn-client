import Combine
import AppSettings
import CountriesManager
import TunnelMixnet
import Tunnels
import TunnelStatus

public final class ConnectionManager: ObservableObject {
    private let appSettings: AppSettings
    private let countriesManager: CountriesManager
    private let tunnelsManager: TunnelsManager

    private var cancellables = Set<AnyCancellable>()

    public static let shared = ConnectionManager()

    @Published public var isTunnelManagerLoaded: Result<Void, Error>?
    @Published public var currentTunnel: Tunnel? {
        didSet {
            guard let currentTunnel else { return }
            configureTunnelStatusObserver(tunnel: currentTunnel)
        }
    }
    @Published public var currentTunnelStatus: TunnelStatus?
    @Published public var entryGateway: EntryGateway? = .randomLowLatency
    @Published public var exitRouter: ExitRouter?

    public init(
        appSettings: AppSettings = AppSettings.shared,
        countriesManager: CountriesManager = CountriesManager.shared,
        tunnelsManager: TunnelsManager = TunnelsManager.shared
    ) {
        self.appSettings = appSettings
        self.countriesManager = countriesManager
        self.tunnelsManager = tunnelsManager
        setup()
    }

    public func connectDisconnect(with config: MixnetConfig) {
        if let activeTunnel = currentTunnel,
           activeTunnel.status == .connected || activeTunnel.status == .connecting {
            disconnect(tunnel: activeTunnel)
        } else {
            connectMixnet(with: config)
        }
    }
}

// MARK: - Setup -
private extension ConnectionManager {
    func setup() {
        setupTunnelManagerObservers()
        setupCountriesManagerObserver()
    }

    func setupTunnelManagerObservers() {
        tunnelsManager.$isLoaded.sink { [weak self] isLoaded in
            self?.isTunnelManagerLoaded = isLoaded
        }
        .store(in: &cancellables)

        tunnelsManager.$activeTunnel.sink { [weak self] tunnel in
            self?.currentTunnel = tunnel
        }
        .store(in: &cancellables)
    }

    func configureTunnelStatusObserver(tunnel: Tunnel) {
        tunnel.$status.sink { [weak self] status in
            self?.currentTunnelStatus = status
        }
        .store(in: &cancellables)
    }
}

// MARK: - Tunnel config -
private extension ConnectionManager {
    func addMixnetConfigurationAndConnect(with config: MixnetConfig) {
        tunnelsManager.add(tunnelConfiguration: config) { [weak self] result in
            switch result {
            case .success(let tunnel):
                self?.currentTunnel = tunnel
                self?.tunnelsManager.connect(tunnel: tunnel)
            case .failure(let error):
                // TODO: handle error
                print("Error: \(error)")
            }
        }
    }
}

// MARK: - Connection -
private extension ConnectionManager {
    func connectMixnet(with config: MixnetConfig) {
        if let tunnel = tunnelsManager.tunnels.first(where: { $0.name == config.name }) {
            currentTunnel = tunnel
            tunnelsManager.connect(tunnel: tunnel)
        } else {
            addMixnetConfigurationAndConnect(with: config)
        }
    }

    func connectWireguard() {}

    func disconnect(tunnel: Tunnel) {
        tunnelsManager.disconnect(tunnel: tunnel)
    }
}

// MARK: - Countries -

private extension ConnectionManager {
    func setupCountriesManagerObserver() {
        countriesManager.$entryCountries.sink { [weak self] _ in
            self?.updateCountries()
        }
        .store(in: &cancellables)

        countriesManager.$exitCountries.sink { [weak self] _ in
            self?.updateCountries()
        }
        .store(in: &cancellables)

        countriesManager.$lowLatencyCountry.sink { [weak self] _ in
            self?.updateCountries()
        }
        .store(in: &cancellables)
    }

    func updateCountries() {
        Task { @MainActor in
            if appSettings.isEntryLocationSelectionOn {
                updateCountriesEntryExit()
            } else {
                updateCountriesExitOnly()
            }
        }
    }

    func updateCountriesEntryExit() {
        guard
            let exitCountries = countriesManager.exitCountries,
            let firstExitCountry = exitCountries.first,
            let lowLatencyCountry = countriesManager.lowLatencyCountry
        else {
            return
        }
        exitRouter = .country(code: firstExitCountry.code)
        entryGateway = .lowLatencyCountry(code: lowLatencyCountry.code)
    }

    func updateCountriesExitOnly() {
        guard let lowLatencyCountry = countriesManager.lowLatencyCountry
        else {
            exitRouter = .randomLowLatency
            return
        }
        entryGateway = .randomLowLatency
        exitRouter = .lowLatencyCountry(code: lowLatencyCountry.code)
    }
}
