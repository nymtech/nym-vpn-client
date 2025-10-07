import Foundation
import Combine
import NymLogger
import SwiftUI
import ErrorReason
import NymVPNRpc
import Logging
import AppVersionProvider
import Constants
import TunnelStatus

@MainActor
public final class GRPCManager: ObservableObject {
    public static let shared = GRPCManager()

    let logger = Logger(label: "GRPC Manager")

    var rpcClient: RpcClient?
    private var listenToEventsObserver: StreamObserver?
    var versionPingTask: Task<Void, Never>?

    @Published public var isServing = false
    @Published public var tunnelStatus: TunnelStatus = .unknown
    @Published public var errorReason: Error?
    @Published public var connectedDate: Date?
    @Published public var connectionRetryAttempt: Int?
    @Published public var tunnelConnectingState: TunnelConnectingState?
    @Published public var connectionInfoData: ConnectionInfoData?
    @Published public var networkName: String?
    @Published public var daemonVersion = "unknown"

    public var requiredVersion: String { AppVersionProvider.libVersion }

    public var requiresUpdate: Bool {
        let required = daemonVersion.semVerCore
        let current  = AppVersionProvider.libVersion.semVerCore
        return required.compare(current, options: .numeric) == .orderedAscending
    }

    public var userAgent: UserAgent {
        UserAgent(
            application: AppVersionProvider.app,
            version: "\(AppVersionProvider.appVersion()) (\(daemonVersion))",
            platform: AppVersionProvider.platform,
            gitCommit: ""
        )
    }

    private init() {
        setup()
    }

    func setup() {
        Task { @MainActor in
            try? await configureRpcClient()
            await pingDaemonInitialStatus()
        }
    }
}

private extension GRPCManager {
    func configureRpcClient() async throws {
        do {
            rpcClient = try await RpcClient()
        } catch {
            logger.error("Failed to create RpcClient: \(error.localizedDescription)")
            return
        }

        listenToEventsObserver = try await rpcClient?.listenToEvents(observer: self)

        stopInitialStatusPinger()
        startDaemonInitialStatusPingerIfNeeded()
    }
}

// MARK: - Helpers
private extension String {
    /// Keep only the first three "."-separated segments (e.g. "1.9.0-beta")
    var semVerCore: String {
        let parts = split(separator: ".")
        guard parts.count >= 3 else { return self }
        return parts[0...2].joined(separator: ".")
    }
}
