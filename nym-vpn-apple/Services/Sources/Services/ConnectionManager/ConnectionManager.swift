import Combine
import Foundation
import NetworkExtension
import AppSettings
import CountriesManager
import CredentialsManager
import TunnelMixnet
import Tunnels
import TunnelStatus
#if os(macOS)
import GRPCManager
#endif

public final class ConnectionManager: ObservableObject {
    private let appSettings: AppSettings
    private let connectionStorage: ConnectionStorage
    private let countriesManager: CountriesManager
    private let tunnelsManager: TunnelsManager
    private let credentialsManager: CredentialsManager

#if os(macOS)
    private let grpcManager: GRPCManager
#endif

    private var cancellables = Set<AnyCancellable>()
    public var isReconnecting = false

    public static let shared = ConnectionManager()

    @Published public var connectionType: ConnectionType {
        didSet {
            appSettings.connectionType = connectionType.rawValue
            reconnectIfNeeded()
        }
    }
    @Published public var isTunnelManagerLoaded: Result<Void, Error>?
#if os(iOS)
    @Published public var currentTunnel: Tunnel? {
        didSet {
            guard let currentTunnel else { return }
            configureTunnelStatusObserver(tunnel: currentTunnel)
        }
    }
#endif
    @Published public var currentTunnelStatus: TunnelStatus? {
        didSet {
            guard isReconnecting,
                  currentTunnelStatus == .disconnected
            else {
                return
            }
            isReconnecting = false
            try? connectDisconnect()
        }
    }
    @Published public var entryGateway: EntryGateway {
        didSet {
            guard entryGateway.isCountry else { return }
            appSettings.entryCountryCode = entryGateway.countryCode ?? "CH"
            reconnectIfNeeded()
        }
    }
    @Published public var exitRouter: ExitRouter {
        didSet {
            guard exitRouter.isCountry else { return }
            appSettings.exitCountryCode = exitRouter.countryCode ?? "CH"
            reconnectIfNeeded()
        }
    }

#if os(iOS)
    public init(
        appSettings: AppSettings = AppSettings.shared,
        connectionStorage: ConnectionStorage = ConnectionStorage.shared,
        countriesManager: CountriesManager = CountriesManager.shared,
        credentialsManager: CredentialsManager = CredentialsManager.shared,
        tunnelsManager: TunnelsManager = TunnelsManager.shared
    ) {
        self.appSettings = appSettings
        self.connectionStorage = connectionStorage
        self.countriesManager = countriesManager
        self.credentialsManager = credentialsManager
        self.tunnelsManager = tunnelsManager
        self.entryGateway = connectionStorage.entryGateway()
        self.exitRouter = connectionStorage.exitRouter()
        self.connectionType = connectionStorage.connectionType()
        setup()
    }
#endif
#if os(macOS)
    public init(
        appSettings: AppSettings = AppSettings.shared,
        connectionStorage: ConnectionStorage = ConnectionStorage.shared,
        countriesManager: CountriesManager = CountriesManager.shared,
        credentialsManager: CredentialsManager = CredentialsManager.shared,
        tunnelsManager: TunnelsManager = TunnelsManager.shared,
        grpcManager: GRPCManager = GRPCManager.shared
    ) {
        self.appSettings = appSettings
        self.connectionStorage = connectionStorage
        self.countriesManager = countriesManager
        self.credentialsManager = credentialsManager
        self.tunnelsManager = tunnelsManager
        self.grpcManager = grpcManager
        self.entryGateway = connectionStorage.entryGateway()
        self.exitRouter = connectionStorage.exitRouter()
        self.connectionType = connectionStorage.connectionType()
        setup()
    }
#endif

#if os(iOS)
    public func isReconnecting(newConfig: MixnetConfig) -> Bool {
        guard let tunnelProviderProtocol = currentTunnel?.tunnel.protocolConfiguration as? NETunnelProviderProtocol,
              let mixnetConfig = tunnelProviderProtocol.asMixnetConfig(),
              currentTunnelStatus == .connected, newConfig != mixnetConfig
        else {
            return false
        }
        return true
    }

    // Reconnect after connection type, hop change
    public func reconnectIfNeeded() {
        guard currentTunnelStatus == .connected else { return }
        try? connectDisconnect()
    }

    public func connectDisconnect() throws {
        do {
            let credentialURL = try credentialsManager.dataFolderURL()
            var config = MixnetConfig(
                entryGateway: entryGateway,
                exitRouter: exitRouter,
                credentialsDataPath: credentialURL.path()
            )

            switch connectionType {
            case .mixnet5hop:
                config = MixnetConfig(
                    entryGateway: entryGateway,
                    exitRouter: exitRouter,
                    credentialsDataPath: credentialURL.path(),
                    isTwoHopEnabled: false
                )
            case .mixnet2hop:
                config = MixnetConfig(
                    entryGateway: entryGateway,
                    exitRouter: exitRouter,
                    credentialsDataPath: credentialURL.path(),
                    isTwoHopEnabled: true
                )
            case .wireguard:
                break
            }
            isReconnecting = isReconnecting(newConfig: config)

            if isReconnecting,
               let activeTunnel = currentTunnel,
               activeTunnel.status == .connected || activeTunnel.status == .connecting {
                disconnect(tunnel: activeTunnel)
            } else {
                if let activeTunnel = currentTunnel,
                   activeTunnel.status == .connected || activeTunnel.status == .connecting {
                    disconnect(tunnel: activeTunnel)
                } else {
                    connectMixnet(with: config)
                }
            }
        } catch let error {
            throw error
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

// MARK: - Connection -
#if os(iOS)
private extension ConnectionManager {
    func connectMixnet(with config: MixnetConfig) {
        tunnelsManager.addUpdate(tunnelConfiguration: config) { [weak self] result in
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
        entryGateway = connectionStorage.entryGateway()
        exitRouter = connectionStorage.exitRouter()
    }

    func updateCountriesExitOnly() {
        entryGateway = connectionStorage.entryGateway()
        exitRouter = connectionStorage.exitRouter()
    }
}
