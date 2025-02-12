import SecurityFoundation
import ServiceManagement
import AppVersionProvider
import GRPCManager
import Shell

// Any changes made to Info.plist & Launchd.plist - are used to create daemon in nym-vpnd.

public final class HelperManager {
    private let grpcManager: GRPCManager
    private let daemon = SMAppService.daemon(plistName: "net.nymtech.vpn.helper.plist")

    private var pollingTask: Task<Void, Never>?
    private var isInstalledAndUpToDate: Bool {
        daemon.status == .enabled && !grpcManager.requiresUpdate && grpcManager.isServing
    }

    public static let shared = HelperManager()

    public var requiredVersion: String {
        grpcManager.requiredVersion
    }
    public var currentVersion: String {
        grpcManager.daemonVersion
    }

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

    public func install() throws {
        do {
            switch daemon.status {
            case .notRegistered, .notFound:
                try? daemon.register()
                try install()
            case .enabled:
                return
            case .requiresApproval:
                SMAppService.openSystemSettingsLoginItems()
            @unknown default:
                break
            }
        } catch {
            daemonState = .unknown
            throw error
        }
    }

    public func uninstall() async throws {
        do {
            try await daemon.unregister()
            try await Task.sleep(for: .seconds(1))
            updateDaemonState()
        }
    }

    public func update() throws {
        daemonState = .updating
        Task(priority: .background) {
            do {
                try await uninstall()
                try daemon.register()
                try await Task.sleep(for: .seconds(1))
                Task { @MainActor [weak self] in
                    self?.daemonState = .running
                }
            } catch {
                Task { @MainActor [weak self] in
                    self?.daemonState = .running
                }
                throw error
            }
        }
    }
}

// MARK: - Private -
private extension HelperManager {
    func setup() {
        updateDaemonState()
        starPolling()
    }

    func updateDaemonState() {
        guard daemonState != .updating else { return }

        switch daemon.status {
        case .notRegistered, .notFound:
            daemonState = .unknown
        case .enabled:
            daemonState = isInstalledAndUpToDate ? .running : .requiresUpdate
        case .requiresApproval:
            daemonState = .requiresAuthorization
        @unknown default:
            break
        }
    }

    func isHelperRunning() -> Bool {
        guard let output = Shell.exec(command: Command.isHelperRunning), !output.isEmpty
        else {
            return false
        }
        return true
    }
}

// MARK: - Polling -
private extension HelperManager {
    func starPolling() {
        pollingTask = Task(priority: .background) { [weak self] in
            guard let self else { return }
            while pollingTask != nil {
                updateDaemonState()
                try? await Task.sleep(for: .seconds(2))
            }
        }
    }
}
