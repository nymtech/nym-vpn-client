import NetworkExtension
import SwiftUI
import Logging
import TunnelStatus

public final class Tunnel: NSObject, ObservableObject {
    public var name: String
    public var tunnel: NETunnelProviderManager
    @Published public var status: TunnelStatus

    private var logger: Logger

    public var onDemandEnabled: Bool {
        tunnel.isEnabled && tunnel.isOnDemandEnabled
    }
    public var containsOnDemandRules: Bool {
        !(tunnel.onDemandRules ?? []).isEmpty
    }

    public init(tunnel: NETunnelProviderManager) {
        self.name = tunnel.localizedDescription ?? "Unnamed"
        self.tunnel = tunnel
        self.status = TunnelStatus(from: tunnel.connection.status)
        self.logger = Logger(label: "Tunnel \(name)")
    }

    func connect(recursionCount: UInt = 0, lastError: Error? = nil) async throws {
        if recursionCount >= 8 {
            logger.log(level: .error, "Connecting failed after 8 attempts. Last error: \(String(describing: lastError))")
            if let lastError {
                throw lastError
            } else {
                return
            }
        }

        logger.log(level: .info, "Connecting tunnel \(name)")

        status = .connecting // Ensure that no other tunnel can attempt activation until this tunnel is done trying

        guard tunnel.isEnabled
        else {
            // Re-enable the tunnel and save it
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

        // Start the tunnel
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
                try? await connect(recursionCount: recursionCount + 1, lastError: systemError)
            } else {
                throw error
            }
        }
    }

    func disconnect() {
        logger.log(level: .info, "Disconnecting Tunnel \(name)")
        tunnel.connection.stopVPNTunnel()
    }

    func updateStatus() {
        status = TunnelStatus(from: tunnel.connection.status)
    }
}
