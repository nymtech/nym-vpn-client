import Combine
import AppSettings
import CountriesManager
import TunnelMixnet
import Tunnels
import TunnelStatus
#if os(macOS)
import GRPCManager
#endif

public final class ConnectionManager: ObservableObject {
    private let appSettings: AppSettings
    private let countriesManager: CountriesManager
    private let tunnelsManager: TunnelsManager
#if os(macOS)
    private let grpcManager: GRPCManager
#endif

    private var cancellables = Set<AnyCancellable>()

    public static let shared = ConnectionManager()

    @Published public var isTunnelManagerLoaded: Result<Void, Error>?
#if os(iOS)
    @Published public var currentTunnel: Tunnel? {
        didSet {
            guard let currentTunnel else { return }
            configureTunnelStatusObserver(tunnel: currentTunnel)
        }
    }
#endif
    @Published public var currentTunnelStatus: TunnelStatus?
    @Published public var entryGateway: EntryGateway? = .randomLowLatency
    @Published public var exitRouter: ExitRouter?

#if os(iOS)
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
#endif
#if os(macOS)
    public init(
        appSettings: AppSettings = AppSettings.shared,
        countriesManager: CountriesManager = CountriesManager.shared,
        tunnelsManager: TunnelsManager = TunnelsManager.shared,
        grpcManager: GRPCManager = GRPCManager.shared
    ) {
        self.appSettings = appSettings
        self.countriesManager = countriesManager
        self.tunnelsManager = tunnelsManager
        self.grpcManager = grpcManager
        setup()
    }
#endif

#if os(iOS)
    public func connectDisconnect(with config: MixnetConfig) {
        if let activeTunnel = currentTunnel,
           activeTunnel.status == .connected || activeTunnel.status == .connecting {
            disconnect(tunnel: activeTunnel)
        } else {
            connectMixnet(with: config)
        }
    }
#endif

#if os(macOS)
    public func connectDisconnect(with config: MixnetConfig) {
        if grpcManager.tunnelStatus == .connected || grpcManager.tunnelStatus == .connecting {
            grpcManager.disconnect()
        } else {
            grpcManager.connect(
                entryGatewayCountryCode: config.entryGateway?.countryCode,
                exitRouterCountryCode: config.exitRouter.countryCode,
                isTwoHopEnabled: config.isTwoHopEnabled
            )
        }
    }
#endif
}

// MARK: - Setup -
#if os(iOS)
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
#endif

#if os(macOS)
private extension ConnectionManager {
    func setup() {
//        setupTunnelManagerObservers()
        setupCountriesManagerObserver()
    }
}
#endif
// MARK: - Tunnel config -
#if os(iOS)
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
#endif

// MARK: - Connection -
#if os(iOS)
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
#endif
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
            let firstExitCountry = exitCountries.first
        else {
            // TODO: get country
            exitRouter = .country(code: "AU")
            return
        }
        if let unwrappedExitRouter = exitRouter, !unwrappedExitRouter.isCountry {
            exitRouter = .country(code: firstExitCountry.code)
        }
    }

    func updateCountriesExitOnly() {
        guard let country = countriesManager.exitCountries?.first
        else {
            exitRouter = .random
            return
        }

        entryGateway = .randomLowLatency
        exitRouter = .country(code: country.code)
    }
}
