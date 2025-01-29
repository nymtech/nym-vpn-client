import Combine
import Foundation
import NetworkExtension
import AppSettings
import CountriesManager
import ConnectionTypes
import CredentialsManager
import NotificationMessages
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
    private var tunnelStatusUpdateCancellable: AnyCancellable?

    public var isReconnecting = false
    public var isDisconnecting = false

    public static let shared = ConnectionManager()

    @Published public var lastError: Error?

    @Published public var connectionType: ConnectionType {
        didSet {
            appSettings.connectionType = connectionType.rawValue
            Task { @MainActor in
                await reconnectIfNeeded()
            }
        }
    }
    @Published public var isTunnelManagerLoaded: Result<Void, Error>?
#if os(iOS)
    @Published public var activeTunnel: Tunnel? {
        didSet {
            guard let activeTunnel else { return }
            configureTunnelStatusObserver(tunnel: activeTunnel)
        }
    }
#endif
    @Published public var currentTunnelStatus: TunnelStatus = .disconnected {
        didSet {
            updateTunnelStatusIfReconnecting()
            updateTunnelStatusIfDisconnecting()
        }
    }
    @Published public var entryGateway: EntryGateway {
        didSet {
            Task { @MainActor in
                connectionStorage.entryGateway = entryGateway
                await reconnectIfNeeded()
            }
        }
    }
    @Published public var exitRouter: ExitRouter {
        didSet {
            Task { @MainActor in
                connectionStorage.exitRouter = exitRouter
                await reconnectIfNeeded()
            }
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
        self.entryGateway = connectionStorage.entryGateway
        self.exitRouter = connectionStorage.exitRouter
        self.connectionType = connectionStorage.connectionType
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
        self.entryGateway = connectionStorage.entryGateway
        self.exitRouter = connectionStorage.exitRouter
        self.connectionType = connectionStorage.connectionType
        setup()
    }
#endif

#if os(iOS)
    public func isReconnecting(newConfig: MixnetConfig) -> Bool {
        guard let tunnelProviderProtocol = activeTunnel?.tunnel.protocolConfiguration as? NETunnelProviderProtocol,
              let mixnetConfig = tunnelProviderProtocol.asMixnetConfig(),
              currentTunnelStatus == .connected, newConfig != mixnetConfig
        else {
            return false
        }
        return true
    }

    /// connects disconnects VPN, depending on current VPN status
    /// - Parameter isAutoConnect: Bool. 
    /// true - when reconnecting automatically, after change of connection settings:  country(UK, DE) or type(5hop, 2hop...).
    /// false - when user manually taps "Connect".
    /// On reconnect, after disconnect, the connectDisconnect is called as a user tapped connect.
    public func connectDisconnect(isAutoConnect: Bool = false) async throws {
        do {
            let config = try generateConfig()
            if isReconnecting {
                // Reconnecting after change of country, 5hop...
                disconnectActiveTunnel()
            } else {
                // User "Connect" button actions
                guard !isAutoConnect else { return }
                if shouldDisconnectActiveTunnel() {
                    isDisconnecting = true
                    disconnectActiveTunnel()
                } else {
                    try await connect(with: config)
                }
            }
        } catch let error {
            throw error
        }
    }
#endif

#if os(macOS)
    public func isReconnecting(newConfig: MixnetConfig) -> Bool {
        if currentTunnelStatus == .connected,
           let lastConfig = MixnetConfig.from(jsonString: appSettings.lastConnectionIntent ?? ""),
           lastConfig != newConfig {
            return true
        } else {
            return false
        }
    }

    public func connectDisconnect(isAutoConnect: Bool = false) async throws {
        let config = generateConfig()
        if isReconnecting {
            // Reconnecting after change of country, 5hop...
            grpcManager.disconnect()
        } else {
            // User "Connect" button actions
            guard !isAutoConnect else { return }
            if grpcManager.tunnelStatus == .connected || grpcManager.tunnelStatus == .connecting {
                isDisconnecting = true
                grpcManager.disconnect()
            } else {
                Task { @MainActor in
                    appSettings.lastConnectionIntent = config.toJson()
                }
                try await grpcManager.connect(
                    entryGateway: config.entryGateway,
                    exitRouter: config.exitRouter,
                    isTwoHopEnabled: config.isTwoHopEnabled,
                    isZknymEnabled: appSettings.isZknymEnabled
                )
            }
        }
    }
#endif

    /// Disconnects tunnel if connected.
    /// iOS removes tunnel profile.
    public func disconnectBeforeLogout() async {
        guard currentTunnelStatus != .disconnected else { return }
#if os(iOS)
        disconnectActiveTunnel()
        await waitForTunnelStatus(with: .disconnected)
        resetVpnProfile()
#elseif os(macOS)
        grpcManager.disconnect()
        await waitForTunnelStatus(with: .disconnected)
#endif
    }
}

// MARK: - Setup -
private extension ConnectionManager {
    func setup() {
#if os(iOS)
        setupTunnelManagerObservers()
#endif

#if os(macOS)
        setupGRPCManagerObservers()
#endif
        setupCountriesManagerObserver()
        setupConnectionChangeObserver()
        setupConnectionErrorObserver()
    }
}

#if os(iOS)
private extension ConnectionManager {
    func setupTunnelManagerObservers() {
        tunnelsManager.$isLoaded.sink { [weak self] isLoaded in
            self?.isTunnelManagerLoaded = isLoaded
        }
        .store(in: &cancellables)

        tunnelsManager.$activeTunnel.sink { [weak self] tunnel in
            self?.activeTunnel = tunnel
        }
        .store(in: &cancellables)
    }

    func configureTunnelStatusObserver(tunnel: Tunnel) {
        tunnelStatusUpdateCancellable = tunnel.$status.sink { [weak self] status in
            self?.currentTunnelStatus = status
        }
    }
}
#endif

#if os(macOS)
private extension ConnectionManager {
    func setupGRPCManagerObservers() {
        grpcManager.$tunnelStatus.sink { [weak self] status in
            guard self?.currentTunnelStatus != status else { return }
            self?.currentTunnelStatus = status
            self?.scheduleNotificationIfNeeded()
        }
        .store(in: &cancellables)
    }

    func scheduleNotificationIfNeeded() {
        guard currentTunnelStatus == .disconnecting else { return }
        Task(priority: .background) {
            await NotificationMessages.scheduleDisconnectNotification()
        }
    }
}
#endif

// MARK: - Reset VPN profile -
public extension ConnectionManager {
    func resetVpnProfile() {
        tunnelsManager.resetVpnProfile()
    }
}

// MARK: - Connection -
#if os(iOS)
private extension ConnectionManager {
    func connect(with config: MixnetConfig) async throws {
        do {
            try await tunnelsManager.loadTunnels()
            let tunnel = try await tunnelsManager.addUpdate(tunnelConfiguration: config)
            activeTunnel = tunnel
            try await tunnelsManager.connect(tunnel: tunnel)
        } catch {
            throw error
        }
    }

    func disconnectActiveTunnel() {
        guard let activeTunnel,
              shouldDisconnectActiveTunnel()
        else {
            return
        }
        tunnelsManager.disconnect(tunnel: activeTunnel)
    }

    func shouldDisconnectActiveTunnel() -> Bool {
        guard let activeTunnel else { return false }

        switch activeTunnel.status {
        case .connected, .connecting, .reasserting, .restarting:
            return true
        case .disconnecting, .disconnected:
            return false
        }
    }

    func generateConfig() throws -> MixnetConfig {
        do {
            let credentialURL = try credentialsManager.dataFolderURL()
            var config = MixnetConfig(
                entryGateway: entryGateway,
                exitRouter: exitRouter,
                credentialsDataPath: credentialURL.path(),
                isZknymEnabled: appSettings.isZknymEnabled
            )

            switch connectionType {
            case .mixnet5hop:
                config = MixnetConfig(
                    entryGateway: entryGateway,
                    exitRouter: exitRouter,
                    credentialsDataPath: credentialURL.path(),
                    isTwoHopEnabled: false,
                    isZknymEnabled: appSettings.isZknymEnabled
                )
            case .wireguard:
                    config = MixnetConfig(
                        entryGateway: entryGateway,
                        exitRouter: exitRouter,
                        credentialsDataPath: credentialURL.path(),
                        isTwoHopEnabled: true,
                        isZknymEnabled: appSettings.isZknymEnabled
                    )
            }
            isReconnecting = isReconnecting(newConfig: config)
            return config
        } catch let error {
            throw error
        }
    }
}
#endif

#if os(macOS)
extension ConnectionManager {
    func generateConfig() -> MixnetConfig {
        var config = MixnetConfig(
            entryGateway: entryGateway,
            exitRouter: exitRouter,
            isZknymEnabled: appSettings.isZknymEnabled
        )

        switch connectionType {
        case .mixnet5hop:
            config = MixnetConfig(
                entryGateway: entryGateway,
                exitRouter: exitRouter,
                isTwoHopEnabled: false,
                isZknymEnabled: appSettings.isZknymEnabled
            )
        case .wireguard:
                config = MixnetConfig(
                    entryGateway: entryGateway,
                    exitRouter: exitRouter,
                    isTwoHopEnabled: true,
                    isZknymEnabled: appSettings.isZknymEnabled
                )
        }
        isReconnecting = isReconnecting(newConfig: config)
        return config
    }
}
#endif

private extension ConnectionManager {
    // Reconnect after connection type, hop change
    func reconnectIfNeeded() async {
        guard currentTunnelStatus == .connected else { return }
        try? await connectDisconnect(isAutoConnect: true)
    }

    func updateTunnelStatusIfReconnecting() {
        guard isReconnecting,
              currentTunnelStatus == .disconnected
        else {
            return
        }
        isReconnecting = false
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) { [weak self] in
            Task {
                try? await self?.connectDisconnect()
            }
        }
    }

    func updateTunnelStatusIfDisconnecting() {
        guard isDisconnecting,
              currentTunnelStatus == .disconnected
        else {
            return
        }
        isDisconnecting = false
    }

    func waitForTunnelStatus(with targetStatus: TunnelStatus) async {
        await withCheckedContinuation { continuation in
            var cancellable: AnyCancellable?

            cancellable = $currentTunnelStatus
                .sink { status in
                    guard cancellable != nil,
                        status == targetStatus
                    else {
                        return
                    }
                    continuation.resume()
                    cancellable?.cancel()
                    cancellable = nil
                }
        }
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

        countriesManager.$vpnCountries.sink { [weak self] _ in
            self?.updateCountries()
        }
        .store(in: &cancellables)
    }

    func setupConnectionChangeObserver() {
        $connectionType.sink { [weak self] _ in
            self?.updateCountries()
        }
        .store(in: &cancellables)
    }

    func setupConnectionErrorObserver() {
#if os(iOS)
        tunnelsManager.$lastError.sink { [weak self] newError in
            self?.lastError = newError
        }
        .store(in: &cancellables)
#endif

#if os(macOS)
        grpcManager.$lastError.sink { [weak self] newError in
            self?.lastError = newError
        }
        .store(in: &cancellables)
#endif
    }

    func updateCountries() {
        Task { @MainActor in
            updateConnectionHops()
        }
    }

    func updateConnectionHops() {
        entryGateway = connectionStorage.entryGateway
        exitRouter = connectionStorage.exitRouter
    }
}
