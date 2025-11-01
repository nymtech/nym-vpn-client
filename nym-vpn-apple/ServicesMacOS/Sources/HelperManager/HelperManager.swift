import Combine
import SecurityFoundation
import ServiceManagement
import AppVersionProvider
import GRPCManager
import Logging
import NymLogger

// Any changes made to Info.plist - are used to create daemon in nym-vpnd.

@MainActor public final class HelperManager: ObservableObject {
    public static let shared = HelperManager(grpcManager: .shared)

    private let grpcManager: GRPCManager
    private let daemon = SMAppService.daemon(plistName: "net.nymtech.vpn.helper.plist")
    private let daemonUpdater = SMAppService.daemon(plistName: "net.nymtech.vpn.updater.plist")
    private var cancellables = Set<AnyCancellable>()
    private var pollingTask: Task<Void, Never>?

    private var isInstalledAndUpToDate: Bool {
        daemon.status == .enabled && !grpcManager.requiresUpdate && grpcManager.isServing
    }

    let logger = Logger(label: "🚜 HelperManager")

    @Published public var daemonState = DaemonState.unknown

    public init(grpcManager: GRPCManager) {
        self.grpcManager = grpcManager
        setup()
    }

    // MARK: - Public API

    public func isInstallNeeded() -> Bool {
        // If .connected, no need to perform install checks to be able to disconnect
        guard grpcManager.tunnelStatus != .connected, !isInstalledAndUpToDate else { return false }
        return true
    }

    public func uninstall() async throws {
        try await daemon.unregister()
        try await daemonUpdater.unregister()
        try await Task.sleep(for: .seconds(1))
        updateDaemonState()
    }

    public func openSystemSettings() {
        SMAppService.openSystemSettingsLoginItems()
    }

    public func requiresDaemonMigration() -> Bool {
        let url = URL(fileURLWithPath: "/Library/LaunchDaemons/net.nymtech.vpn.helper.plist")
        let legacyStatus = SMAppService.statusForLegacyPlist(at: url)
        return legacyStatus == .enabled || legacyStatus == .requiresApproval
    }

    public func registerDaemonIfNeeded() {
        do {
            switch daemon.status {
            case .notRegistered, .notFound:
                try daemon.register()
            default:
                break
            }
        } catch {
            print("Failed to register daemon: \(error)")
            logger.error("Failed to register daemon: \(error)")
        }
    }

    public func registerDaemonUpdaterIfNeeded() {
        do {
            switch daemonUpdater.status {
            case .notRegistered, .notFound:
                try daemonUpdater.register()
            default:
                break
            }
        } catch {
            print("Failed to register daemon updater: \(error)")
            logger.error("Failed to register daemon: \(error)")
        }
    }
}

// MARK: - Private
private extension HelperManager {

    func setup() {
        updateDaemonState()
        setupGrpcManagerObservers()
        registerDaemonUpdaterIfNeeded()
        registerDaemonIfNeeded()
        try? updateDaemonIfNeeded()
    }

    func setupGrpcManagerObservers() {
        grpcManager.$daemonVersion
            .removeDuplicates()
            .sink { [weak self] _ in
                self?.updateDaemonState()
            }
            .store(in: &cancellables)

        grpcManager.$tunnelStatus
            .removeDuplicates()
            .sink { [weak self] newTunnelStatus in
                guard let self else { return }
                guard newTunnelStatus != .connected else { return }
                try? self.updateDaemonIfNeeded()
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
            guard grpcManager.isServing else {
                checkIfDaemonNeedsForcedUpdate()
                return
            }

            if grpcManager.daemonVersion != "unknown" && grpcManager.daemonVersion != "noVersion" {
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
            startPolling()
        } else {
            pollingTask?.cancel()
            pollingTask = nil
            try? updateDaemonIfNeeded()
        }

        guard newState != daemonState else { return }
        daemonState = newState
        logger.info("State changed to: \(newState)")
    }

    // Version 2.6.0, change of vpn service update
    func checkIfDaemonNeedsForcedUpdate() {
        if grpcManager.daemonVersion == "update" {
            daemonState = .requiresUpdate
            try? updateDaemonIfNeeded()
        } else {
            daemonState = .unknown
        }
    }

    func updateDaemonIfNeeded() throws {
        guard daemonState == .requiresUpdate, grpcManager.tunnelStatus != .connected else { return }
        daemonState = .updating

        Task { @MainActor in
            do {
                logger.info("Update if needed...")
                logger.info("daemonState: \(self.daemonState)")
                logger.info("Req. v: \(AppVersionProvider.libVersion)")
                logger.info("Cur. v: \(self.grpcManager.daemonVersion)")

                logger.info("Updating...")
                callKillHelper()
                logger.info("Updated")

                try await Task.sleep(for: .seconds(3))
                self.daemonState = .running
            } catch {
                // Fall back to updating state + re-evaluating
                self.daemonState = .running
                self.updateDaemonState()
                throw error
            }
        }
    }
}

// MARK: - Polling
private extension HelperManager {
    func startPolling() {
        pollingTask?.cancel()
        pollingTask = Task { [weak self] in
            guard let self else { return }
            while !Task.isCancelled, self.pollingTask != nil {
                self.updateDaemonState()
                try? await Task.sleep(for: .seconds(5))
            }
        }
    }
}
