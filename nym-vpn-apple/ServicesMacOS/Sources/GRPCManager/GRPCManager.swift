import Foundation
import NymLogger
import SwiftUI
import Base58Swift
import GRPC
import ErrorReason
import Logging
import NIO
import NIOConcurrencyHelpers
import SwiftProtobuf
import AppVersionProvider
import Constants
import TunnelStatus

public final class GRPCManager: ObservableObject {
    private let group = MultiThreadedEventLoopGroup(numberOfThreads: System.coreCount)
    private let channel: GRPCChannel
    private let unixDomainSocket = "/var/run/nym-vpn.sock"

    let client: NymVpnService_NymVpnServiceAsyncClient
    let logger = Logger(label: "GRPC Manager")

    var userAgent: NymVpnService_UserAgent {
        var agent = NymVpnService_UserAgent()
        agent.application = AppVersionProvider.app
        agent.version = "\(AppVersionProvider.appVersion()) (\(daemonVersion))"
        agent.platform = AppVersionProvider.platform
        return agent
    }
    var versionPingTask: Task<Void, Never>?

    public static let shared = GRPCManager()

    @Published public var isServing = false
    @Published public var tunnelStatus: TunnelStatus = .unknown
    @Published public var errorReason: Error?
    @Published public var connectedDate: Date?
    @Published public var connectionRetryAttempt: Int?
    @Published public var tunnelConnectingState: TunnelConnectingState?
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
        channel = ClientConnection(
            configuration:
                    .default(
                        target: .unixDomainSocket(unixDomainSocket),
                        eventLoopGroup: group
                    )
        )

        client = NymVpnService_NymVpnServiceAsyncClient(channel: channel)
        setup()
    }

    deinit {
        try? channel.close().wait()
        try? group.syncShutdownGracefully()
        stopInitialStatusPinger()
    }

    func setup() {
        setupListenToTunnelStateChangesObserver()
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
