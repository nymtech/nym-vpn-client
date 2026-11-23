import NetworkExtension
import Logging
import ConfigurationManager
import Constants
import NymLogger
import ErrorHandler
import NymVPNLib
import TunnelMixnet
import Tunnels
import AppVersionProvider

final class PacketTunnelProvider: NEPacketTunnelProvider, @unchecked Sendable {
    let tunnelActor: TunnelActor
    let logger: Logger

    override init() {
        tunnelActor = TunnelActor()

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
        logger = Logger(label: "PacketTunnelProvider")

        super.init()
    }

    override func startTunnel(options: [String: NSObject]? = nil) async throws {
        await tunnelActor.setTunnelProvider(self)

        guard let tunnelProviderProtocol = protocolConfiguration as? NETunnelProviderProtocol,
              let mixnetConfig = await tunnelProviderProtocol.asMixnetConfig()
        else {
            logger.error("Failed to obtain tunnel configuration")
            throw PacketTunnelProviderError.invalidSavedConfiguration
        }
        // Configured here rather than in init() so Sentry follows the user's error-reporting
        // opt-in, which only arrives with the tunnel configuration.
        configureLogger(isErrorReportingEnabled: mixnetConfig.isErrorReportingEnabled)

        let vpnConfig = try mixnetConfig.asVpnConfig(tunProvider: self)
        try await setup(vpnConfig: vpnConfig)

        try await ensureGatewayIndependenceAllowsConnect(
            remindersEnabled: mixnetConfig.isServerFamilyRemindersEnabled
        )

        _ = try await tunnelActor.withCommandSender { try await $0.connectTunnel() }
        try await tunnelActor.waitUntilStarted()
    }

    override func wake() {
        // Lets the core quietly redial the bridge transport instead of waiting to notice on its
        // own next scheduled probe, which can be minutes away after a long device sleep.
        notifyDeviceWoke()
    }

    override func stopTunnel(with reason: NEProviderStopReason) async {
        logger.info("Stop tunnel... \(reason.rawValue)")

        await tunnelActor.cancelRelaxConsent()

        if reason == .userInitiated {
            await tunnelActor.clearError()
        }
        await tunnelActor.shutdownVpnService()
        await tunnelActor.setTunnelProvider(nil)
    }
}

extension PacketTunnelProvider {
    func ensureGatewayIndependenceAllowsConnect(remindersEnabled: Bool) async throws {
        let needsConsent = try await tunnelActor.withCommandSender { commandSender in
            try await commandSender.setEnableGatewayIndependence(enableGatewayIndependence: true)
            guard case .needsRelaxedIndependenceCriteria = try await commandSender.getTentativeGateways()
            else {
                return false
            }

            guard remindersEnabled
            else {
                try await commandSender.setEnableGatewayIndependence(enableGatewayIndependence: false)
                return false
            }

            return true
        }
        guard needsConsent == true else { return }

        await tunnelActor.reportNeedsRelaxedIndependence()
        try await tunnelActor.awaitRelaxConsent()
        await tunnelActor.clearError()
    }

    func setup(vpnConfig: VpnConfig) async throws {
        try await ConfigurationManager.shared.setup()

        let vpnService = try await NymVpnService.newService(
            config: vpnConfig,
            environment: ConfigurationManager.shared.networkEnv ?? .newWithMainnetFallback(),
            eventListener: self
        )
        await tunnelActor.setVpnService(vpnService)
    }

    /// `initLogger` is one-shot per process: if the extension is reused for another start,
    /// the first call's settings stay in effect.
    private func configureLogger(isErrorReportingEnabled: Bool) {
        let logFilePath = LogFileManager.logFileURL(logFileType: .library)?.path()
        // Extracted from ConfigurationManager.shared.debugLevel
        let isTestFlight = Bundle.main.appStoreReceiptURL?.lastPathComponent == "sandboxReceipt"
        let isDebugLogsOn = UserDefaults(
            suiteName: Constants.groupID.rawValue
        )?.bool(forKey: "isDebugLogsOn") ?? false

        let logLevel: LogLevel = (isTestFlight || isDebugLogsOn) ? .debug : .info
        initLogger(logFilePath: logFilePath, logLevel: logLevel, sentryMonitoring: isErrorReportingEnabled)
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
            throw VpnErrorUniFFIBoundary.vpnError(from: error)
        }
    }
}
