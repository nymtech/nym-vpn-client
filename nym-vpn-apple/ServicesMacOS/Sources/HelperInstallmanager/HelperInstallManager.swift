import SwiftUI
import GRPCManager
import HelperManager

public final class HelperInstallManager: ObservableObject {
    private let helperManager: HelperManager
    private let grpcManager: GRPCManager

    private var isInstalledAndUpToDate: Bool {
        let result = !grpcManager.requiresUpdate && grpcManager.isServing
        if result {
            daemonState = .running
        }
        return result
    }

    public static let shared = HelperInstallManager(helperManager: .shared, grpcManager: .shared)

    @Published public var daemonState = DaemonState.unknown

    public init(helperManager: HelperManager, grpcManager: GRPCManager) {
        self.helperManager = helperManager
        self.grpcManager = grpcManager
    }
}

extension HelperInstallManager {
    public func installIfNeeded() async throws {
        // If tunnelStatus == .connected, do not perform install checks.
        // No need to reinstall daemon before disconnect.
        guard grpcManager.tunnelStatus != .connected, !isInstalledAndUpToDate else { return }

        daemonState = .installing
        do {
            try await helperManager.installHelper()
        } catch {
            daemonState = .unknown
            throw error
        }
        try await Task.sleep(for: .seconds(10))
        daemonState = .installed
    }
}
