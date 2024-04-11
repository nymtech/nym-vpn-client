import NetworkExtension
import Logging
import SwiftUI
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

    func connect(recursionCount: UInt = 0, lastError: Error? = nil) {
        if recursionCount >= 8 {
            logger.log(level: .error, "Connecting failed after 8 attempts. Last error: \(String(describing: lastError))")
            return
        }

        logger.log(level: .debug, "Connecting tunnel \(name)")

        status = .connecting // Ensure that no other tunnel can attempt activation until this tunnel is done trying

        guard tunnel.isEnabled
        else {
            // In case the tunnel had gotten disabled, re-enable and save it,
            // then call this function again.
            logger.log(level: .debug, "Connecting. Enabling tunnel.")
            tunnel.isEnabled = true
            tunnel.saveToPreferences { [weak self] error in
                guard let self = self else { return }
                if error != nil {
                    logger.log(
                        level: .error,
                        "Connecting. Error saving tunnel after re-enabling: \(String(describing: error))"
                    )
                    return
                }
                logger.log(level: .debug, "Connecting Tunnel saved after re-enabling, invoking connect")
                self.connect(recursionCount: recursionCount + 1, lastError: NEVPNError(NEVPNError.configurationUnknown))
            }
            return
        }

        // Start the tunnel
        do {
            logger.log(level: .debug, "Connecting starting tunnel...")
            //            isAttemptingActivation = true
            //            let activationAttemptId = UUID().uuidString
            //            self.activationAttemptId = activationAttemptId
            //            try (tunnelProvider.connection as? NETunnelProviderSession)?.startTunnel(options: ["activationAttemptId": activationAttemptId])
            try tunnel.connection.startVPNTunnel()
            logger.log(level: .debug, "Connecting starting tunnel success")
            status = TunnelStatus(from: tunnel.connection.status)
        } catch let error {
            //            isAttemptingActivation = false
            guard let systemError = error as? NEVPNError else {
                logger.log(level: .error, "Failed to start tunnel: Error: \(error)")
                status = .disconnected
                return
            }
            guard systemError.code == .configurationInvalid || systemError.code == .configurationStale
            else {
                logger.log(level: .error, "Failed to start tunnel: Error: \(error)")
                status = .disconnected
                return
            }
            logger.log(level: .debug, "Connecting - reloading tunnel")
            tunnel.loadFromPreferences { [weak self] error in
                guard let self = self else { return }
                if error != nil {
                    logger.log(level: .error, "Connecting. Error reloading tunnel: \(String(describing: error))")
                    self.status = .disconnected
                    return
                }
                logger.log(level: .debug, "Connecting - reconnecting")
                self.connect(recursionCount: recursionCount + 1, lastError: systemError)
            }
        }
    }

    func disconnect() {
        logger.log(level: .debug, "Disconnecting Tunnel \(name)")
        tunnel.connection.stopVPNTunnel()
    }

    func updateStatus() {
        status = TunnelStatus(from: tunnel.connection.status)
    }
}
