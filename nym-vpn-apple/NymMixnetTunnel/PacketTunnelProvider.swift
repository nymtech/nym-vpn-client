import NetworkExtension
import Logging
import NymLogger
import MixnetLibrary
import TunnelMixnet
import Tunnels

class PacketTunnelProvider: NEPacketTunnelProvider {
    private lazy var mixnetTunnelProvider = MixnetTunnelProvider()
    private lazy var mixnetAdapter: MixnetAdapter = {
        MixnetAdapter(
            with: self,
            mixnetTunnelProvider: mixnetTunnelProvider
        )
    }()
    private lazy var logger = Logger(label: "MixnetTunnel")

    override init() {
        LoggingSystem.bootstrap { label in
            FileLogHandler(label: label)
        }
    }

    override func startTunnel(options: [String: NSObject]? = nil) async throws {
        logger.log(level: .info, "Start tunnel...")
        guard
            let tunnelProviderProtocol = self.protocolConfiguration as? NETunnelProviderProtocol,
            let mixnetConfig = tunnelProviderProtocol.asMixnetConfig()
        else {
            logger.log(level: .info, "Start tunnel: invalid saved configuration")
            throw PacketTunnelProviderError.invalidSavedConfiguration
        }

        let callback: () -> Void = { [weak self] in
            guard let config = self?.mixnetTunnelProvider.nymConfig
            else {
                return
            }

            self?.configure(with: config)
            self?.mixnetTunnelProvider.fileDescriptor = self?.mixnetAdapter.tunnelFileDescriptor
            self?.logger.log(
                level: .info,
                "Start tunnel: \(String(describing: self?.mixnetAdapter.tunnelFileDescriptor))"
            )
        }
        mixnetTunnelProvider.nymOnConfigure = callback

        do {
            self.logger.log(level: .info, "Start tunnel: start")
            try mixnetAdapter.start(
                with: mixnetConfig.asVpnConfig(mixnetTunnelProvider: mixnetAdapter.mixnetTunnelProvider)
            )
        } catch let error {
            logger.log(level: .error, "Start tunnel: \(error)")
            throw error
        }
        logger.log(level: .info, "Start tunnel: connected")
    }

    override func stopTunnel(with reason: NEProviderStopReason) async {
        do {
            try mixnetAdapter.stop()
            logger.log(level: .error, "Stop tunnel reason: \(reason)")
        } catch let error {
            logger.log(level: .error, "Stop tunnel reason: \(reason), error: \(error)")
        }
    }
}

private extension PacketTunnelProvider {
    func configure(with config: NymConfig) {
        let networkSettings = MixnetTunnelSettingsGenerator(nymConfig: config).generateNetworkSettings()
        do {
            try setNetworkSettings(networkSettings)
        } catch {
            logger.log(level: .error, "Configure error: \(error)")
        }
    }

    func setNetworkSettings(_ networkSettings: NEPacketTunnelNetworkSettings) throws {
        var systemError: Error?
        let condition = NSCondition()

        condition.lock()
        defer { condition.unlock() }

        setTunnelNetworkSettings(networkSettings) { error in
            systemError = error
            condition.signal()
        }

        let setTunnelNetworkSettingsTimeout: TimeInterval = 5 // seconds

        if !condition.wait(until: Date().addingTimeInterval(setTunnelNetworkSettingsTimeout)) {
            logger.log(level: .error, "setTunnelNetworkSettings timed out")
        }

        if let error = systemError {
            throw error
        }
    }
}
