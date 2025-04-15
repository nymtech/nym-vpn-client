import Combine
import SecurityFoundation
import ServiceManagement
import AppVersionProvider
import GRPCManager
import Shell

// Any changes made to Info.plist & Launchd.plist - are used to create daemon in nym-vpnd.

public final class HelperManager: ObservableObject {
    private let grpcManager: GRPCManager
    private let daemon = SMAppService.daemon(plistName: "net.nymtech.vpn.helper.plist")

    private var cancellables = Set<AnyCancellable>()
    private var pollingTask: Task<Void, Never>?
    private var isInstalledAndUpToDate: Bool {
        daemon.status == .enabled && !grpcManager.requiresUpdate && grpcManager.isServing
    }

    public static let shared = HelperManager()

    @Published public var daemonState = DaemonState.unknown

    public init(grpcManager: GRPCManager = .shared) {
        self.grpcManager = grpcManager
        setup()
    }

    public func isInstallNeeded() -> Bool {
        // If .connected, no need to perform install checks to be able to disconnect
        guard grpcManager.tunnelStatus != .connected, !isInstalledAndUpToDate else { return false }
        return true
    }

    public func uninstall() async throws {
        do {
            try await daemon.unregister()
            try await Task.sleep(for: .seconds(1))
            updateDaemonState()
        }
    }

    public func openSystemSettings() {
        SMAppService.openSystemSettingsLoginItems()
    }

    public func requiresDaemonMigration() -> Bool {
        let url = URL(fileURLWithPath: "/Library/LaunchDaemons/net.nymtech.vpn.helper.plist")
        let legacyStatus = SMAppService.statusForLegacyPlist(at: url)
        return legacyStatus == .enabled || legacyStatus == .requiresApproval
    }
}

// MARK: - Private -
private extension HelperManager {
    func setup() {
        updateDaemonState()
        setupGrpcManagerObservers()
        registerDaemonIfNeeded()
        try? updateDaemonIfNeeded()
    }

    func setupGrpcManagerObservers() {
        grpcManager.$daemonVersion
            .receive(on: DispatchQueue.main)
            .removeDuplicates()
            .sink { [weak self] _ in
                self?.updateDaemonState()
            }
            .store(in: &cancellables)

        grpcManager.$tunnelStatus
            .receive(on: DispatchQueue.main)
            .removeDuplicates()
            .sink { [weak self] newTunnelStatus in
                guard newTunnelStatus != .connected else { return }
                try? self?.updateDaemonIfNeeded()
            }
            .store(in: &cancellables)
    }

    func updateDaemonState() {
        guard daemonState != .updating else { return }
        var newState: DaemonState

        switch daemon.status {
        case .notRegistered, .notFound:
            newState = .unknown
        case .enabled:
            if grpcManager.daemonVersion != "unknown" || grpcManager.daemonVersion != "noVersion" {
                newState = isInstalledAndUpToDate ? .running : .requiresUpdate
            } else {
                newState = .authorized
            }
        case .requiresApproval:
            newState = .requiresAuthorization
        @unknown default:
            newState = .unknown
        }

        if requiresDaemonMigration() {
            newState = .requiresManualRemoval
            starPolling()
        } else {
            pollingTask?.cancel()
            pollingTask = nil
            try? updateDaemonIfNeeded()
        }

        guard newState != daemonState else { return }
        daemonState = newState
    }

    func registerDaemonIfNeeded() {
        do {
            switch daemon.status {
            case .notRegistered, .notFound:
                try? daemon.register()
            default:
                break
            }
        }
    }

    func updateDaemonIfNeeded() throws {
        guard daemonState == .requiresUpdate, grpcManager.tunnelStatus != .connected else { return }
        daemonState = .updating
        Task {
            do {
                try await uninstall()
                try daemon.register()
                try await Task.sleep(for: .seconds(3))
                Task { @MainActor [weak self] in
                    self?.daemonState = .running
                }
            } catch {
                Task { @MainActor [weak self] in
                    self?.daemonState = .running
                    self?.updateDaemonState()
                }
                throw error
            }
        }
    }
}

// MARK: - Polling -
private extension HelperManager {
    func starPolling() {
        pollingTask = Task { [weak self] in
            guard let self else { return }
            while pollingTask != nil {
                updateDaemonState()
                try? await Task.sleep(for: .seconds(5))
            }
        }
    }
}
