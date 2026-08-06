import Foundation
import Combine
import NymLogger
import SwiftUI
import ServiceManagement
import ErrorReason
import NymVPNLib
import Logging
import AppVersionProvider
import Constants
import ConnectionTypes
import TunnelStatus

let RPC_RECONNECT_DELAY = Duration.seconds(5)

@MainActor public final class GRPCManager: ObservableObject {
    private var cancellables = Set<AnyCancellable>()
    private var listenToEventsObserver: StreamObserver?

    public static let shared = GRPCManager()

    let logger = Logger(label: "GRPC Manager")
    // TODO: create actor
    var rpcClient: RpcClient?
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
            let rpcClient = try await RpcClient()
            self.rpcClient = rpcClient

            let newRpcObserver = RPCTunnelObserver()
            listenToEventsObserver = try await rpcClient.listenToEvents(observer: newRpcObserver)

            Task { [weak self] in
                for await event in newRpcObserver.stream {
                    self?.didReceive(event: event)
                }

                await self?.onDisconnect()
            }

            stopInitialStatusPinger()
            startDaemonInitialStatusPingerIfNeeded()
        } catch {
            let error = (error as? RpcError)?.displayChain() ?? error.localizedDescription
            logger.error("Failed to connect RpcClient: \(error)")
            try? await Task.sleep(for: RPC_RECONNECT_DELAY)
            setup()
        }
    }

    func didReceive(event: TunnelEvent) {
        switch event {
        case let .newState(tunnelState):
            Task { @MainActor in
                updateTunnelStatus(with: tunnelState)
            }
        case .mixnetState:
            break
        case .configChanged:
            break
        case .accountState:
            break
        case .diagnosticsSuggested(_):
            break
        }
    }

    func onDisconnect() async {
        logger.warning("🛩️ RPC connection closed")

        isServing = false
        tunnelStatus = .unknown
        listenToEventsObserver = nil
        stopInitialStatusPinger()
        rpcClient = nil

        try? await Task.sleep(for: RPC_RECONNECT_DELAY)
        setup()
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
