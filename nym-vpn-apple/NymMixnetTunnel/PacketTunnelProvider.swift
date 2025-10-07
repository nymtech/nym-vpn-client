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

    override init() {
        Self.configureLogger()
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

        try await startNymVpn(
            credentialDataPath: credentialDataPath,
            vpnConfig: vpnConfig,
            isErrorReportingEnabled: mixnetConfig.isErrorReportingEnabled,
            isStatisticsEnabled: mixnetConfig.isStatisticsEnabled
        )
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

    func startNymVpn(
        credentialDataPath: String,
        vpnConfig: VpnConfig,
        isErrorReportingEnabled: Bool,
        isStatisticsEnabled: Bool
    ) async throws {
        do {
            let config = NymVpnLibConfig(
                dataDir: credentialDataPath,
                credentialMode: nil,
                sentryMonitoring: isErrorReportingEnabled,
                statisticsEnabled: isStatisticsEnabled,
                userAgent: .appUserAgent
            )
            try configureLib(config: config)
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
            try await ConfigurationManager.shared.setup(for: .networkExtension)
        } catch {
            self.logger.error("Failed to set environment: \(error)")
        }
    }

    static func configureLogger() {
        let logPath = LogFileManager.logFileURL(logFileType: .library)?.path()

        Task {
            let logLevel = await MainActor.run { ConfigurationManager.shared.debugLevel }
            await initLogger(
                path: logPath,
                debugLevel: logLevel,
                sentryMonitoring: true
            )
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
