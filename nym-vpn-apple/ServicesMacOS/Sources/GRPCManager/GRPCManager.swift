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

    let healthClient: Grpc_Health_V1_HealthClientProtocol
    let client: Nym_Vpn_NymVpndClientProtocol
    let logger = Logger(label: "GRPC Manager")

    var userAgent: Nym_Vpn_UserAgent {
        var agent = Nym_Vpn_UserAgent()
        agent.application = AppVersionProvider.app
        agent.version = "\(AppVersionProvider.appVersion()) (\(daemonVersion))"
        agent.platform = AppVersionProvider.platform
        return agent
    }
    var isServing = false

    public static let shared = GRPCManager()

    @Published public var tunnelStatus: TunnelStatus = .disconnected
    @Published public var errorReason: Error?
    @Published public var connectedDate: Date?
    @Published public var networkName: String?
    public var daemonVersion = "unknown"
    public var requiredVersion: String {
        AppVersionProvider.libVersion
    }

    public var requiresUpdate: Bool {
        daemonVersion.compare(AppVersionProvider.libVersion, options: .numeric) == .orderedAscending
    }

    private init() {
        channel = ClientConnection(
            configuration:
                    .default(
                        target: .unixDomainSocket(unixDomainSocket),
                        eventLoopGroup: group
                    )
        )
        client = Nym_Vpn_NymVpndNIOClient(channel: channel)
        healthClient = Grpc_Health_V1_HealthNIOClient(channel: channel)
        setup()
    }

    deinit {
        try? channel.close().wait()
        try? group.syncShutdownGracefully()
    }

    func setup() {
        setupListenToTunnelStateChangesObserver()
        Task {
            try? await version()
        }
    }

    // MARK: - Connection -

    public func disconnect() {
        logger.log(level: .info, "Disconnecting")

        let call = client.vpnDisconnect(Google_Protobuf_Empty())

        call.response.whenComplete { result in
            switch result {
            case .success(let response):
                print("Disconnected from VPN: \(response.success)")
            case .failure(let error):
                print("Failed to disconnect from VPN: \(error)")
            }
        }

        do {
            _ = try call.status.wait()
        } catch {
            print("Error waiting for call status: \(error)")
        }
    }
}
