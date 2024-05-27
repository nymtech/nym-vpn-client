import Foundation
import SwiftUI
import Base58Swift
import GRPC
import NIO
import NIOConcurrencyHelpers
import SwiftProtobuf
import TunnelStatus

public final class GRPCManager: ObservableObject {
    private let group = PlatformSupport.makeEventLoopGroup(loopCount: 1)
    private let client: Nym_Vpn_NymVpndClientProtocol
    private let channel: GRPCChannel
    private let unixDomainSocket = "/var/run/nym-vpn.sock"

    public static let shared = GRPCManager()

    @Published public var tunnelStatus: TunnelStatus = .disconnected

    private init() {
        channel = ClientConnection(
            configuration:
                    .default(
                        target: .unixDomainSocket(unixDomainSocket),
                        eventLoopGroup: group
                    )
        )
        client = Nym_Vpn_NymVpndNIOClient(channel: channel)
        setup()
    }

    deinit {
        try? channel.close().wait()
        try? group.syncShutdownGracefully()
    }

    public func status() {
        let request = Nym_Vpn_StatusRequest()
        let call = client.vpnStatus(request)

        call.response.whenComplete { result in
            switch result {
            case let .success(response):
                print("Received response: \(response)")
            case let .failure(error):
                print("Call failed with error: \(error)")
            }
        }

        _ = try? call.status.wait()
    }

    public func importCredential(credential: String) throws {
        var request = Nym_Vpn_ImportUserCredentialRequest()

        guard let base58Array = Base58.base58Decode(credential)
        else {
            throw GRPCError.invalidData
        }
        request.credential = Data(base58Array)

        let call = client.importUserCredential(request)

        var isCredentialImported = false

        call.response.whenComplete { result in
            switch result {
            case .success(let response):
                print("response from daemon: \(response.success)")
                isCredentialImported = response.success
            case .failure(let error):
                isCredentialImported = false
                print("Something went wrong: \(error)")
            }
        }

        do {
            _ = try call.status.wait()
            if !isCredentialImported {
                throw GRPCError.invalidCredential
            }
        }
    }

    public func connect(
        entryGatewayCountryCode: String?,
        exitRouterCountryCode: String?,
        isTwoHopEnabled: Bool
    ) {
        var request = Nym_Vpn_ConnectRequest()

        var entryNode = Nym_Vpn_EntryNode()
        if let entryGatewayCountryCode {
            var location = Nym_Vpn_Location()
            location.twoLetterIsoCountryCode = entryGatewayCountryCode
            entryNode.location = location
        } else {
            entryNode.randomLowLatency = Nym_Vpn_Empty()
        }

        var exitNode = Nym_Vpn_ExitNode()
        if let exitRouterCountryCode {
            var location = Nym_Vpn_Location()
            location.twoLetterIsoCountryCode = exitRouterCountryCode
        } else {
            exitNode.random = Nym_Vpn_Empty()
        }

        request.entry = entryNode
        request.exit = exitNode

        request.disableRouting = false
        request.enableTwoHop = isTwoHopEnabled
        request.enablePoissonRate = false
        request.disableBackgroundCoverTraffic = false
        request.enableCredentialsMode = true

        let call = client.vpnConnect(request)

        call.response.whenComplete { result in
            switch result {
            case .success(let response):
                print("Connected to VPN: \(response.success)")
            case .failure(let error):
                print("Failed to connect to VPN: \(error)")
            }
        }

        do {
            _ = try call.status.wait()
        } catch {
            print("Error waiting for call status: \(error)")
        }
    }

    public func disconnect() {
        let request = Nym_Vpn_DisconnectRequest()

        let call = client.vpnDisconnect(request)

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

private extension GRPCManager {
    func setup() {
        setupListenToConnectionStateObserver()
        setupListenToConnectionStatusObserver()
    }

    func setupListenToConnectionStateObserver() {
        let call = client.listenToConnectionStateChanges(Nym_Vpn_Empty()) { [weak self] connectionStateChange in
            // TODO:
            print("Connection state \(connectionStateChange)")

            switch connectionStateChange.status {
            case .UNRECOGNIZED, .connectionFailed, .notConnected, .statusUnspecified, .unknown:
                self?.tunnelStatus = .disconnected
            case .connecting:
                self?.tunnelStatus = .connecting
            case .connected:
                self?.tunnelStatus = .connected
            case .disconnecting:
                self?.tunnelStatus = .disconnecting
            }
        }

        call.status.whenComplete { result in
            switch result {
            case .success(let status):
                print("Stream completed with status: \(status)")
            case .failure(let error):
                print("Stream failed with error: \(error)")
            }
        }
    }

    func setupListenToConnectionStatusObserver() {
        let call = client.listenToConnectionStatus(Nym_Vpn_Empty()) { connectionStatusUpdate in
            // TODO:
            print("DO ME 2 \(connectionStatusUpdate)")
        }

        call.status.whenComplete { result in
            switch result {
            case .success(let status):
                print("Stream completed with status: \(status)")
            case .failure(let error):
                print("Stream failed with error: \(error)")
            }
        }
    }
}
