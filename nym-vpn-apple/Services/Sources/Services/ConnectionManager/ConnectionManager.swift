import Combine
import Foundation
import NetworkExtension
import os
import AppSettings
import ConnectionTypes
import ConnectionTypes
import CredentialsManager
import TunnelMixnet
import Tunnels
import TunnelStatus
#if os(macOS)
import GRPCManager
#endif

@MainActor public final class ConnectionManager: ObservableObject {
    private static let logoutLogger = Logger(
        subsystem: Bundle.main.bundleIdentifier ?? "NymVPN",
        category: "ConnectionManager.logout"
    )

    private var timerCancellable: AnyCancellable?

    let appSettings: AppSettings
    let connectionStorage: ConnectionStorage
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

    public var isDisconnecting = false

#if os(iOS)
    public static let shared = ConnectionManager(
        appSettings: .shared,
        connectionStorage: .shared,
        credentialsManager: .shared,
        tunnelsManager: .shared
    )
#elseif os(macOS)
    public static let shared = ConnectionManager(
        appSettings: .shared,
        connectionStorage: .shared,
        credentialsManager: .shared,
        tunnelsManager: .shared,
        grpcManager: .shared
    )
#endif

    @Published public var connectionConfig: ConnectionConfig
    @Published public var connectedDate: Date?
    @Published public var connectionRetryAttempt: Int?
    @Published public var afterDisconnectAction: AfterDisconnectAction?
    @Published public var lastError: Error?
    @Published public var tunnelConnectingState: TunnelConnectingState?
    @Published public var connectionInfoData: ConnectionInfoData?

    @Published public var connectionType: ConnectionType
    public var entryGatewayType: NodeType { connectionType == .wireguard ? .vpn : .entry }
    public var exitGatewayType: NodeType { connectionType == .wireguard ? .vpn : .exit }
    @Published public var isTunnelManagerLoaded: Result<Void, Error>?
#if os(iOS)
    @Published public var activeTunnel: Tunnel? {
        didSet {
            guard let activeTunnel else { return }
            configureTunnelStatusObserver(tunnel: activeTunnel)
        }
    }
#endif
    @Published public var currentTunnelStatus: TunnelStatus = .disconnected
    @Published public var entryGateway: EntryGateway
    @Published public var exitRouter: ExitRouter

#if os(iOS)
    public init(
        appSettings: AppSettings,
        connectionStorage: ConnectionStorage,
        credentialsManager: CredentialsManager,
        tunnelsManager: TunnelsManager
    ) {
        self.appSettings = appSettings
        self.connectionStorage = connectionStorage
        self.credentialsManager = credentialsManager
        self.tunnelsManager = tunnelsManager
        self.entryGateway = connectionStorage.entryGateway
        self.exitRouter = connectionStorage.exitRouter
        self.connectionType = connectionStorage.connectionType
        self.connectionConfig = connectionStorage.connectionConfig
        setup()
        setupMockObserverIfNeeded()
    }
#endif

#if os(macOS)
    public init(
        appSettings: AppSettings,
        connectionStorage: ConnectionStorage,
        credentialsManager: CredentialsManager,
        tunnelsManager: TunnelsManager,
        grpcManager: GRPCManager
    ) {
        self.appSettings = appSettings
        self.connectionStorage = connectionStorage
        self.credentialsManager = credentialsManager
        self.tunnelsManager = tunnelsManager
        self.grpcManager = grpcManager
        self.entryGateway = connectionStorage.entryGateway
        self.exitRouter = connectionStorage.exitRouter
        self.connectionType = connectionStorage.connectionType
        self.connectionConfig = connectionStorage.connectionConfig
        setup()
        setupMockObserverIfNeeded()
    }
#endif

    public var isMockModeEnabled: Bool { MockMode.isEnabled }

    /// Disconnects tunnel if connected.
    /// iOS removes tunnel profile when disconnect completes within the logout wait cap.
    public func disconnectBeforeLogout() async {
        let disconnectedInTime = await disconnectForLogout()
#if os(iOS)
        if LogoutTeardownPolicy.shouldResetVpnProfileAfterLogoutDisconnect(
            disconnectedInTime: disconnectedInTime
        ) {
            resetVpnProfile()
        } else {
            Self.logoutLogger.warning(
                "Logout disconnect wait timed out; skipping VPN profile reset"
            )
        }
#endif
        setEntryGateway(.auto)
        setExitGateway(.auto)
    }

    /// Logout path: bounded wait when the user already started disconnecting elsewhere.
    @discardableResult
    func disconnectForLogout() async -> Bool {
        guard LogoutTeardownPolicy.needsDisconnectWait(for: currentTunnelStatus) else { return true }
#if os(iOS)
        if LogoutTeardownPolicy.shouldInitiateDisconnect(for: currentTunnelStatus) {
            try? await disconnectActiveTunnel()
        }
        let disconnectedInTime = await waitForTunnelStatus(
            with: .disconnected,
            timeout: LogoutTeardownPolicy.disconnectWaitCapSeconds
        )
        if !disconnectedInTime {
            Self.logoutLogger.warning(
                "Logout disconnect wait timed out before tunnel reached disconnected"
            )
        }
        return disconnectedInTime
#elseif os(macOS)
        try? await grpcManager.disconnect()
        return await waitForTunnelStatus(
            with: .disconnected,
            timeout: LogoutTeardownPolicy.disconnectWaitCapSeconds
        )
#endif
    }

    /// Disconnect and wait for disconnected status
    public func disconnectAndWaitForDisconnected() async {
        guard currentTunnelStatus != .disconnected else { return }
#if os(iOS)
        try? await disconnectActiveTunnel()
        await waitForTunnelStatus(with: .disconnected)

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
#if SANTA
        registerForEnvironmentChanges()
#endif
    }

    func setupMockObserverIfNeeded() {
        guard MockMode.isEnabled else { return }
        MockConnectionState.shared.$tunnelStatus
            .receive(on: DispatchQueue.main)
            .sink { [weak self] status in
                MainActor.assumeIsolated {
                    self?.currentTunnelStatus = status
                }
            }
            .store(in: &cancellables)
    }
}

// MARK: - Reset VPN profile -
public extension ConnectionManager {
    func resetVpnProfile() {
        tunnelsManager.resetVpnProfile()
#if os(iOS)
        MixnetConfigStorage.delete()
#endif
    }
}

// MARK: - Connection -

extension ConnectionManager {
    @discardableResult
    func waitForTunnelStatus(with targetStatus: TunnelStatus, timeout: TimeInterval? = nil) async -> Bool {
        if currentTunnelStatus == targetStatus { return true }

        if let timeout {
            let pollInterval: Duration = .milliseconds(250)
            let deadline = ContinuousClock.now + .seconds(timeout)
            while ContinuousClock.now < deadline {
                if currentTunnelStatus == targetStatus { return true }
                try? await Task.sleep(for: pollInterval)
            }
            return currentTunnelStatus == targetStatus
        }

        await waitForTunnelStatusChange(to: targetStatus)
        return currentTunnelStatus == targetStatus
    }

    private func waitForTunnelStatusChange(to targetStatus: TunnelStatus) async {
        if currentTunnelStatus == targetStatus { return }

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
// MARK: - Setup -

private extension ConnectionManager {
    func setupConnectionChangeObserver() {
        $connectionType.sink { [weak self] _ in
            self?.updateCountries()
        }
        .store(in: &cancellables)

        $connectionConfig.sink { [weak self] newConnectionConfig in
            self?.connectionStorage.connectionConfig = newConnectionConfig
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

