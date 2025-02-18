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

    public static let shared = GRPCManager()

    var userAgent: Nym_Vpn_UserAgent {
        var agent = Nym_Vpn_UserAgent()
        agent.application = AppVersionProvider.app
        agent.version = "\(AppVersionProvider.appVersion()) (\(daemonVersion))"
        agent.platform = AppVersionProvider.platform
        return agent
    }

    @Published public var tunnelStatus: TunnelStatus = .disconnected
    @Published public var errorReason: ErrorReason?
    @Published public var generalError: GeneralNymError?
    @Published public var connectedDate: Date?
    @Published public var isServing = false
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
        setupHealthObserver()
        setupListenToTunnelStateChangesObserver()
    }

    // MARK: - Info -

    public func version() async throws {
        logger.log(level: .info, "Version")
        return try await withCheckedThrowingContinuation { continuation in
            Task {
                let call = client.info(Google_Protobuf_Empty(), callOptions: CallOptions(timeLimit: .timeout(.seconds(5))))

                call.response.whenComplete { [weak self] result in
                    switch result {
                    case let .success(response):
                        self?.daemonVersion = response.version
                        self?.networkName = response.nymNetwork.networkName
                        self?.logger.info("🛜 \(response.nymNetwork.networkName)")
                        continuation.resume()
                    case let .failure(error):
                        continuation.resume(throwing: error)
                    }
                }
            }
        }
    }

    // MARK: - Connection -
    public func isReadyToConnect() {
        logger.log(level: .info, "isReadyToConnect")

        let call = client.isReadyToConnect(Google_Protobuf_Empty())
        call.response.whenComplete { [weak self] result in
            switch result {
            case .success(let response):
                self?.logger.log(level: .info, "\(response)")
            case .failure(let error):
                self?.logger.log(level: .info, "Failed to connect to VPN: \(error)")
            }
        }
    }

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

    // MARK: - Countries -
    public func entryCountryCodes() async throws -> [String] {
        logger.log(level: .info, "Fetching entry countries")
        return try await withCheckedThrowingContinuation { continuation in
            var request = Nym_Vpn_ListCountriesRequest()
            request.kind = .mixnetEntry
            request.userAgent = userAgent

            let call = client.listCountries(request, callOptions: nil)
            call.response.whenComplete { result in
                switch result {
                case let .success(countries):
                    continuation.resume(returning: countries.countries.map { $0.twoLetterIsoCountryCode })
                case let .failure(error):
                    continuation.resume(throwing: error)
                }
            }

            call.status.whenComplete { [weak self] result in
                switch result {
                case .success:
                    break
                case let .failure(error):
                    self?.logger.log(level: .error, "\(error.localizedDescription)")
                }
            }
        }
    }

    public func exitCountryCodes() async throws -> [String] {
        logger.log(level: .info, "Fetching exit countries")
        return try await withCheckedThrowingContinuation { continuation in
            var request = Nym_Vpn_ListCountriesRequest()
            request.kind = .mixnetExit
            request.userAgent = userAgent

            let call = client.listCountries(request, callOptions: nil)
            call.response.whenComplete { result in
                switch result {
                case let .success(countries):
                    continuation.resume(returning: countries.countries.map { $0.twoLetterIsoCountryCode })
                case let .failure(error):
                    continuation.resume(throwing: error)
                }
            }

            call.status.whenComplete { [weak self] result in
                switch result {
                case .success:
                    break
                case let .failure(error):
                    self?.logger.log(level: .error, "\(error.localizedDescription)")
                }
            }
        }
    }

    public func vpnCountryCodes() async throws -> [String] {
        logger.log(level: .info, "Fetching VPN countries")
        return try await withCheckedThrowingContinuation { continuation in
            var request = Nym_Vpn_ListCountriesRequest()
            request.kind = .wg
            request.userAgent = userAgent

            let call = client.listCountries(request, callOptions: nil)
            call.response.whenComplete { result in
                switch result {
                case let .success(countries):
                    continuation.resume(returning: countries.countries.map { $0.twoLetterIsoCountryCode })
                case let .failure(error):
                    continuation.resume(throwing: error)
                }
            }

            call.status.whenComplete { [weak self] result in
                switch result {
                case .success:
                    break
                case let .failure(error):
                    self?.logger.log(level: .error, "\(error.localizedDescription)")
                }
            }
        }
    }
}
