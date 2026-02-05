import NetworkExtension
import Logging
import ConfigurationManager
import NymLogger
import ErrorHandler
import NymVPNLib
import TunnelMixnet
import Tunnels
import AppVersionProvider

class PacketTunnelProvider: NEPacketTunnelProvider {
    let tunnelActor: TunnelActor

    lazy var logger = Logger(label: "MixnetTunnel")
    var logInitFailure: String?
    var vpnService: NymVpnService?
    var commandSender: NymVpnServiceCommandSender?

    override init() {
        tunnelActor = TunnelActor()
        super.init()

        self.configureLogger()
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
    }

    override func startTunnel(options: [String: NSObject]? = nil) async throws {
        await tunnelActor.setTunnelProvider(self)

        guard let tunnelProviderProtocol = protocolConfiguration as? NETunnelProviderProtocol,
              let mixnetConfig = await tunnelProviderProtocol.asMixnetConfig()
        else {
            logger.error("Failed to obtain tunnel configuration")
            throw PacketTunnelProviderError.invalidSavedConfiguration
        }
        let vpnConfig = try mixnetConfig.asVpnConfig(tunProvider: self)
        try await setup(vpnConfig: vpnConfig)

        _ = try await commandSender?.connectTunnel()
    }

    override func stopTunnel(with reason: NEProviderStopReason) async {
        logger.info("Stop tunnel... \(reason.rawValue)")

        do {
            _ = try await commandSender?.disconnectTunnel()
            await vpnService?.shutdownAndWait()
        } catch {
            logger.error("Failed to stop the tunnel: \(error)")
        }

        await tunnelActor.setTunnelProvider(nil)
        vpnService = nil
        commandSender = nil
    }
}

extension PacketTunnelProvider {
    func setup(vpnConfig: VpnConfig) async throws {
        try await ConfigurationManager.shared.setup()

        vpnService = try await NymVpnService.newService(
            config: vpnConfig,
            environment: ConfigurationManager.shared.networkEnv,
            eventListener: self
        )
        commandSender = vpnService?.getCommandSender()
    }

    func configureLogger() {
        let logDir = LogFileManager.logsDirectory()?.path()
        // Extracted from ConfigurationManager.shared.debugLevel
        let isTestFlight = Bundle.main.appStoreReceiptURL?.lastPathComponent == "sandboxReceipt"
        let logLevel: LogLevel = isTestFlight ? .debug : .info
        initLogger(logDir: logDir, logLevel: logLevel, sentryMonitoring: true)
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
