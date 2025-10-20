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
            let newRpcObserver = RPCTunnelObserver()
            listenToEventsObserver = try await rpcClient?.listenToEvents(observer: newRpcObserver)

            newRpcObserver.$didClose
                .receive(on: DispatchQueue.main)
                .sink { [weak self] value in
                    guard value else { return }
                    self?.tunnelStatus = .unknown
                    self?.listenToEventsObserver = nil
                    self?.stopInitialStatusPinger()
                    self?.rpcClient = nil
                    self?.setup()
                }
                .store(in: &cancellables)

            newRpcObserver.$tunnelEvent
                .removeDuplicates()
                .sink { [weak self] value in
                    guard let value else { return }
                    self?.didReceive(event: value)
                }
                .store(in: &cancellables)

            stopInitialStatusPinger()
            startDaemonInitialStatusPingerIfNeeded()
        } catch {
            try? await Task.sleep(for: .seconds(3))
            setup()
            logger.error("Failed to create RpcClient: \(error.localizedDescription)")
            return
        }
    }

    func didReceive(event: TunnelEvent) {
        switch event {
        case let .newState(tunnelState):
            Task { @MainActor in
                updateTunnelStatus(with: tunnelState)
            }
        case .mixnetState:
            Task { @MainActor in }
        case .configChanged:
            Task { @MainActor in }
        case .accountState:
            Task { @MainActor in }
        }
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
