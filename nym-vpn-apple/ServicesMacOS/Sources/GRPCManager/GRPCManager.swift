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

let kRpcReconnectDelay = Duration.seconds(5)

@MainActor public final class GRPCManager: ObservableObject {
    private var cancellables = Set<AnyCancellable>()
    private var eventObserver: StreamObserver?

    public static let shared = GRPCManager()

    let logger = Logger(label: "GRPC Manager")
    var rpcClient: RpcClient?

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
        startConnectionLoop()
    }

    func startConnectionLoop() {
        Task { @MainActor in
            while !Task.isCancelled {
                do {
                    try await connectRpcAndHandleEvents()
                } catch {
                    let error = (error as? RpcError)?.displayChain() ?? error.localizedDescription
                    logger.error("RPC error: \(error)")
                }

                try await Task.sleep(for: kRpcReconnectDelay)
            }
        }
    }
}

private extension GRPCManager {
    func connectRpcAndHandleEvents() async throws {
        let rpcClient = try await RpcClient()
        let newRpcObserver = RPCTunnelObserver()

        eventObserver = try await rpcClient.listenToEvents(observer: newRpcObserver)
        self.rpcClient = rpcClient

        do {
            try await onConnect(rpcClient: rpcClient)
        } catch {
            await onDisconnect()
            throw error
        }

        for await event in newRpcObserver.stream {
            didReceive(event: event)
        }

        await onDisconnect()
    }

    func didReceive(event: TunnelEvent) {
        switch event {
        case let .newState(tunnelState):
            updateTunnelStatus(with: tunnelState)
        case .mixnetState:
            break
        case .configChanged:
            break
        case .accountState:
            break
        case .diagnosticsSuggested:
            break
        case .conflictDetected:
            break
        }
    }

    func onConnect(rpcClient: RpcClient) async throws {
        logger.info("RPC connected")

        let serviceInfo = try await rpcClient.getInfo()
        let tunnelState = try await rpcClient.getTunnelState()

        daemonVersion = serviceInfo.version
        networkName = serviceInfo.nymNetwork.networkName
        logger.info("🛜 \(serviceInfo.nymNetwork.networkName)")

        updateTunnelStatus(with: tunnelState)

        isServing = true
    }

    func onDisconnect() async {
        logger.info("RPC disconnected")

        isServing = false
        tunnelStatus = .unknown
        eventObserver = nil
        rpcClient = nil
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
