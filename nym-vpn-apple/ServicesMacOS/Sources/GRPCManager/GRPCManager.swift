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

public final class GRPCManager: ObservableObject {
    let logger = Logger(label: "GRPC Manager")

    var userAgent: UserAgent {
        UserAgent(
            application: AppVersionProvider.app,
            version: "\(AppVersionProvider.appVersion()) (\(daemonVersion))",
            platform: AppVersionProvider.platform,
            gitCommit: ""
        )
    }
    var rpcClient: RpcClient?
    var listenToEventsObserver: StreamObserver?
    var versionPingTask: Task<Void, Never>?

    public static let shared = GRPCManager()

    @Published public var isServing = false
    @Published public var tunnelStatus: TunnelStatus = .unknown
    @Published public var errorReason: Error?
    @Published public var connectedDate: Date?
    @Published public var connectionRetryAttempt: Int?
    @Published public var tunnelConnectingState: TunnelConnectingState?
    @Published public var connectionInfoData: ConnectionInfoData?
    @Published public var networkName: String?
    @Published public var daemonVersion = "unknown"
    public var requiredVersion: String {
        AppVersionProvider.libVersion
    }

    public var requiresUpdate: Bool {
        let required = daemonVersion.semVerCore
        let current  = AppVersionProvider.libVersion.semVerCore
        return required.compare(current, options: .numeric) == .orderedAscending
    }

    private init() {
        setup()
    }

    func setup() {
        Task {
            try? await configureRpcCLient()
            Task { @MainActor in
                await pingDaemonInitialStatus()
            }
        }
    }

    func configureRpcCLient() async throws {
//        do {
            rpcClient = try await RpcClient()
//        } catch {
//
//        }
        listenToEventsObserver = try await rpcClient?.listenToEvents(observer: RpcTunnelObserver())

        stopInitialStatusPinger()
        startDaemonInitialStatusPingerIfNeeded()
    }
}

private extension String {
    /// Keep only the first three "."-separated segments (e.g. "1.9.0-beta")
    var semVerCore: String {
        let parts = self.split(separator: ".")
        guard parts.count >= 3 else { return self }
        return parts[0...2].joined(separator: ".")
    }
}
