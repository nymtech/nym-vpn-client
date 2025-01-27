import NetworkExtension
import Logging
import ConfigurationManager
import NymLogger
import ErrorHandler
import MixnetLibrary
import TunnelMixnet
import Tunnels

class PacketTunnelProvider: NEPacketTunnelProvider {
    let tunnelActor: TunnelActor

    lazy var logger = Logger(label: "MixnetTunnel")

    override init() {
        let logURL = LogFileManager.logFileURL(logFileType: .library)?.path()
        initLogger(
            path: logURL,
            debugLevel: ConfigurationManager.shared.debugLevel
        )
        LoggingSystem.bootstrap { label in
            let fileLogHandler = FileLogHandler(label: label, logFileManager: LogFileManager(logFileType: .tunnel))
#if DEBUG
            let osLogHandler = OSLogHandler(
                subsystem: Bundle.main.bundleIdentifier ?? "NymMixnetTunnel",
                category: label
            )
            return MultiplexLogHandler([osLogHandler, fileLogHandler])
#else
            return fileLogHandler
#endif
        }

        tunnelActor = TunnelActor()
    }

    override func startTunnel(options: [String: NSObject]? = nil) async throws {
        logger.info("Start tunnel...")

        await setup()

        await tunnelActor.setTunnelProvider(self)

        guard let tunnelProviderProtocol = protocolConfiguration as? NETunnelProviderProtocol,
              let mixnetConfig = tunnelProviderProtocol.asMixnetConfig()
        else {
            logger.error("Failed to obtain tunnel configuration")
            throw PacketTunnelProviderError.invalidSavedConfiguration
        }

        let vpnConfig = try mixnetConfig.asVpnConfig(tunProvider: self, tunStatusListener: self)

        logger.info("Starting backend")

        guard let credentialDataPath = vpnConfig.credentialDataPath
        else {
            throw PacketTunnelProviderError.noCredentialDataDir
        }

        do {
            try await startNymVpn(credentialDataPath: credentialDataPath, vpnConfig: vpnConfig)
        } catch {
            try? stopVpn()
            try? shutdown()
            throw error
        }
    }

    override func stopTunnel(with reason: NEProviderStopReason) async {
        logger.info("Stop tunnel... \(reason.rawValue)")

        do {
            try stopVpn()
        } catch {
            logger.error("Failed to stop the tunnel: \(error)")
        }

        do {
            try shutdown()
        } catch {
            logger.error("Failed to stop account controller: \(error)")
        }

        await tunnelActor.setTunnelProvider(nil)
    }

    func startNymVpn(credentialDataPath: String, vpnConfig: VpnConfig) async throws {
        do {
            try configureLib(dataDir: credentialDataPath, credentialMode: nil)
            try startVpn(config: vpnConfig)
        } catch {
            logger.error("Failed to start vpn: \(error)")
            if let lastVPNError = error as? VpnError {
                throw VPNErrorReason(with: lastVPNError).nsError
            } else {
                throw error
            }
        }

        logger.info("Backend is up and running...")

        do {
            try await tunnelActor.waitUntilStarted()
        } catch {
            logger.error("Failed to wait until vpn started: \(error)")
            throw error
        }
    }
}

extension PacketTunnelProvider {
    func setup() async {
        do {
            try await ConfigurationManager.shared.setup()
        } catch {
            self.logger.error("Failed to set environment: \(error)")
        }
    }
}

extension PacketTunnelProvider: OsTunProvider {
    func setTunnelNetworkSettings(tunnelSettings: TunnelNetworkSettings) async throws {
        do {
            let networkSettings = tunnelSettings.asPacketTunnelNetworkSettings()
            logger.debug("Set network settings: \(networkSettings)")
            try await setTunnelNetworkSettings(networkSettings)
        } catch {
            logger.error("Failed to set tunnel network settings: \(error)")
            throw error
        }
    }
}
