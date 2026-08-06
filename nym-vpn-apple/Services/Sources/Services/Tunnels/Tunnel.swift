import NetworkExtension
import SwiftUI
import Logging
import TunnelStatus

@MainActor public final class Tunnel: NSObject, ObservableObject {
    public var name: String
    public var tunnel: NETunnelProviderManager

    @Published public var status: TunnelStatus
    @Published public var retryAttempt: Int?
    @Published public var afterDisconnectAction: AfterDisconnectAction?
    @Published public var lastError: Error?
    @Published public var tunnelConnectingState: TunnelConnectingState?
    @Published public var connectionInfoData: ConnectionInfoData?

    private var logger: Logger
    private var isPolling = false
    private var pollingTask: Task<Void, Never>?

    public var onDemandEnabled: Bool {
        tunnel.isEnabled && tunnel.isOnDemandEnabled
    }

    public init(tunnel: NETunnelProviderManager) {
        self.name = tunnel.localizedDescription ?? "Unnamed"
        self.tunnel = tunnel
        self.status = TunnelStatus(from: tunnel.connection.status)
        self.logger = Logger(label: "Tunnel \(name)")
        super.init()

        if status != .disconnected {
            startPollingTunnelStatus()
        }
    }

    // MARK: - Actions

    func connect(recursionCount: UInt = 0, lastError: Error? = nil) async throws {
        self.lastError = nil
        startPollingTunnelStatus()

        if recursionCount >= 8 {
            logger.log(level: .error, "Connecting failed after 8 attempts. Last error: \(String(describing: lastError))")
            if let lastError {
                throw lastError
            } else {
                return
            }
        }

        logger.log(level: .info, "Connecting tunnel \(name)")
        status = .connecting

        guard tunnel.isEnabled else {
            logger.log(level: .info, "Connecting. Enabling tunnel.")
            tunnel.isEnabled = true

            do {
                try await tunnel.saveToPreferences()
            } catch {
                self.logger.log(level: .error, "Connecting. Error saving tunnel after re-enabling: \(error)")
                self.status = .disconnected
                throw error
            }

            logger.log(level: .info, "Connecting Tunnel saved after re-enabling, invoking connect")
            try? await connect(
                recursionCount: recursionCount + 1,
                lastError: NEVPNError(NEVPNError.configurationUnknown)
            )
            return
        }

        do {
            logger.log(level: .info, "Connecting starting tunnel...")
            try tunnel.connection.startVPNTunnel()
            logger.log(level: .info, "Connecting starting tunnel success")
            status = TunnelStatus(from: tunnel.connection.status)
        } catch let error {
            logger.log(level: .error, "Failed to start tunnel: \(error)")
            status = .disconnected

            if let systemError = error as? NEVPNError,
               systemError.code == .configurationInvalid || systemError.code == .configurationStale {
                logger.log(level: .info, "Connecting - reloading tunnel")
                do {
                    try await tunnel.loadFromPreferences()
                } catch {
                    logger.log(level: .error, "Connecting. Error reloading tunnel: \(error)")
                    status = .disconnected
                }
                logger.log(level: .info, "Connecting - reconnecting")
                try await connect(recursionCount: recursionCount + 1, lastError: systemError)
            } else {
                throw error
            }
        }
    }

    func disconnect() {
        logger.log(level: .info, "Disconnecting Tunnel \(name)")
        stopPollingTunnelStatus()
        lastError = nil
        tunnel.connection.stopVPNTunnel()
    }

    func updateStatus() {
        status = TunnelStatus(from: tunnel.connection.status)
        if status == .disconnected {
            stopPollingTunnelStatus()
        }
    }

    /// Send a message to the network extension.
    public func send(_ message: TunnelProviderMessage) async throws {
        do {
            try assertCanSendMessage()

            let data = try message.encode()
            _ = try await sendProviderMessage(with: data)
        } catch {
            logger.error("Failed to send tunnel message: \(error.localizedDescription)")
            throw error
        }
    }

    /// Send a message to the network extension decoding response.
    public func sendWithResponse<T: Decodable>(_ message: TunnelProviderMessage) async throws -> T {
        do {
            try assertCanSendMessage()

            if let res = try await sendProviderMessage(with: message.encode()) {
                return try JSONDecoder().decode(T.self, from: res)
            } else {
                throw SendTunnelProviderMessageError.noData
            }
        } catch {
            logger.error("Failed to send tunnel message: \(error.localizedDescription)")
            throw error
        }
    }

    /// Asserts if IPC will be able to pass the message to the network extension.
    private func assertCanSendMessage() throws {
        let status = tunnel.connection.status

        switch status {
        case .connecting, .connected, .reasserting:
            break

        case .invalid, .disconnected, .disconnecting:
            throw SendTunnelProviderMessageError.tunnelDown(status)

        @unknown default:
            break
        }
    }

    private func sendProviderMessage(with messageData: Data) async throws -> Data? {
        if let session = tunnel.connection as? NETunnelProviderSession {
            return try await session.sendProviderMessageAsync(messageData)
        } else {
            logger.warning("TunnelProvider session is nil")
            throw SendTunnelProviderMessageError.noActiveTunnel
        }
    }
}

enum SendTunnelProviderMessageError: LocalizedError {
    /// No data returned by network extension
    case noData

    /// System error
    case system(Error)

    /// No active tunnel is around
    case noActiveTunnel

    /// Tunnel is down or about to go down
    case tunnelDown(NEVPNStatus)

    var errorDescription: String? {
        switch self {
        case .noData:
            return "No data was returned"
        case let .system(err):
            return "System error: \(err)"
        case .noActiveTunnel:
            return "No active tunnel"
        case let .tunnelDown(status):
            return "Tunnel is down or about to go down: \(status)"
        }
    }
}

// MARK: - Manager helper
extension Tunnel {
    public func saveToPreferencesAndLoadTunnels() async throws {
        try await tunnel.savePrefsAndReloadOnMainActor()
    }
}

// MARK: - Polling
private extension Tunnel {
    func startPollingTunnelStatus() {
        isPolling = true
        pollingTask = Task { [weak self] in
            guard let self else { return }
            while self.isPolling {
                await self.pollTunnelStatus()
                try? await Task.sleep(for: .seconds(1))
            }
        }
    }

    func stopPollingTunnelStatus() {
        isPolling = false
        pollingTask?.cancel()
        pollingTask = nil
    }

    func pollTunnelStatus() async {
        do {
            let decoded: TunnelStatusResponse = try await sendWithResponse(.status)

            self.retryAttempt = decoded.retryAttempt
            self.afterDisconnectAction = decoded.afterDisconnectAction
            self.tunnelConnectingState = decoded.tunnelConnectingState
            self.connectionInfoData = decoded.connectionInfoData

            guard self.isPolling else { return }
            if let newError = decoded.lastError, self.status != .error {
                self.status = .error
                self.lastError = newError
            } else if self.status != decoded.status {
                self.status = decoded.status
            }
        } catch {
            logger.error("Failed to poll status: \(error)")
        }
    }
}
