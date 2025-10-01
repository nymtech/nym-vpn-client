import Combine
import Foundation
import NetworkExtension
import AppSettings
import ConnectionTypes
import CredentialsManager
import TunnelMixnet
import Tunnels
import TunnelStatus
#if os(macOS)
import GRPCManager
#endif

public final class ConnectionManager: ObservableObject {
    private let connectionStorage: ConnectionStorage

    private var timerCancellable: AnyCancellable?

    let appSettings: AppSettings
    let credentialsManager: CredentialsManager
    let tunnelsManager: TunnelsManager
#if os(macOS)
    let grpcManager: GRPCManager
#endif

    var cancellables = Set<AnyCancellable>()
    var tunnelStatusUpdateCancellable: AnyCancellable?
    var tunnelRetryAttemptCancellable: AnyCancellable?
    var tunnelAfterRetryCancellable: AnyCancellable?
    var tunnelLastErrorCancelable: AnyCancellable?
    var tunnelConnectingStateCancellable: AnyCancellable?
    var tunnelConnectionInfoDataCancellable: AnyCancellable?

    // TODO: remove this once iOS tunnel supports tunnel reconnection
    public var isReconnecting = false
    public var isDisconnecting = false

    public static let shared = ConnectionManager()

    @Published public var connectionConfig: ConnectionConfig?
    @Published public var connectedDate: Date?
    @Published public var connectedDateString: String?
    @Published public var connectionRetryAttempt: Int?
    @Published public var afterDisconnectAction: AfterDisconnectAction?
    @Published public var lastError: Error?
    @Published public var tunnelConnectingState: TunnelConnectingState?
    @Published public var connectionInfoData: ConnectionInfoData?

    @Published public var connectionType: ConnectionType {
        didSet {
            switch connectionType {
            case .mixnet5hop:
                connectionConfig?.enableTwoHop = false
            case .wireguard:
                connectionConfig?.enableTwoHop = true
            }
            appSettings.connectionType = connectionType.rawValue
            updateConnectionConfig()
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

    // TODO: remove this once iOS tunnel supports tunnel reconnection
    @Published public var currentTunnelStatus: TunnelStatus = .disconnected {
        didSet {
            updateTunnelStatusIfReconnecting()
        }
    }
#elseif os(macOS)
    @Published public var currentTunnelStatus: TunnelStatus = .disconnected
#endif
    @Published public var entryGateway: EntryGateway {
        didSet {
            Task { @MainActor in
                connectionConfig?.entry = entryGateway
                connectionStorage.entryGateway = entryGateway
                updateConnectionConfig()
            }
        }
    }
    @Published public var exitRouter: ExitRouter {
        didSet {
            Task { @MainActor in
                connectionConfig?.exit = exitRouter
                connectionStorage.exitRouter = exitRouter
                updateConnectionConfig()
            }
        }
    }

#if os(iOS)
    public init(
        appSettings: AppSettings = AppSettings.shared,
        connectionStorage: ConnectionStorage = ConnectionStorage.shared,
        credentialsManager: CredentialsManager = CredentialsManager.shared,
        tunnelsManager: TunnelsManager = TunnelsManager.shared
    ) {
        self.appSettings = appSettings
        self.connectionStorage = connectionStorage
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
        credentialsManager: CredentialsManager = CredentialsManager.shared,
        tunnelsManager: TunnelsManager = TunnelsManager.shared,
        grpcManager: GRPCManager = GRPCManager.shared
    ) {
        self.appSettings = appSettings
        self.connectionStorage = connectionStorage
        self.credentialsManager = credentialsManager
        self.tunnelsManager = tunnelsManager
        self.grpcManager = grpcManager
        self.entryGateway = connectionStorage.entryGateway
        self.exitRouter = connectionStorage.exitRouter
        self.connectionType = connectionStorage.connectionType
        setup()
    }
#endif

    /// Disconnects tunnel if connected.
    /// iOS removes tunnel profile.
    public func disconnectBeforeLogout() async {
        guard currentTunnelStatus != .disconnected else { return }
#if os(iOS)
        try? await disconnectActiveTunnel()
        await waitForTunnelStatus(with: .disconnected)
        resetVpnProfile()
#elseif os(macOS)
        try? await grpcManager.disconnect()
        await waitForTunnelStatus(with: .disconnected)
#endif
    }
}

// MARK: - Setup -
private extension ConnectionManager {
    func setup() {
#if os(iOS)
        setupTunnelManagerObservers()
#elseif os(macOS)
        setupGRPCManagerObservers()
#endif
        setupConnectionChangeObserver()
        setupConnectionErrorObserver()
        configureConnectedTimeTimer()
        Task { @MainActor in
            await fetchConnectionConfig()
        }
    }
}

// MARK: - Reset VPN profile -
public extension ConnectionManager {
    func resetVpnProfile() {
        tunnelsManager.resetVpnProfile()
    }
}

// MARK: - Connection -

private extension ConnectionManager {
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
    func setupConnectionChangeObserver() {
        $connectionType.sink { [weak self] _ in
            self?.updateCountries()
        }
        .store(in: &cancellables)
    }

    func setupConnectionErrorObserver() {
#if os(iOS)
        tunnelsManager.$lastError
            .receive(on: DispatchQueue.main)
            .sink { [weak self] newError in
                self?.lastError = newError
            }
            .store(in: &cancellables)
#elseif os(macOS)
        grpcManager.$errorReason
            .receive(on: DispatchQueue.main)
            .sink { [weak self] newError in
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

// MARK: - Connection time -
private extension ConnectionManager {
    func configureConnectedTimeTimer() {
        timerCancellable = Timer.publish(every: 1.0, on: .main, in: .common)
            .autoconnect()
            .receive(on: DispatchQueue.main)
            .sink { [weak self] _ in
                guard let self = self else { return }
                updateConnectedDateString()
            }
    }

    func updateConnectedDateString() {
        guard let connectedDate
        else {
            guard connectedDateString != nil else { return }
            connectedDateString = nil
            return
        }
        let timeElapsed = Date().timeIntervalSince(connectedDate)
        let hours = Int(timeElapsed) / 3600
        let minutes = (Int(timeElapsed) % 3600) / 60
        let seconds = Int(timeElapsed) % 60
        let newConnectedDateString = "\(String(format: "%02d:%02d:%02d", hours, minutes, seconds))"
        guard connectedDateString != newConnectedDateString else { return }
        connectedDateString = newConnectedDateString
    }
}
