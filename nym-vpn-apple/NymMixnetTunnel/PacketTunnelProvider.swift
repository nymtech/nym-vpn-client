import NetworkExtension
import Logging
import NymLogger
import MixnetLibrary
import TunnelMixnet
import Tunnels
import ConfigurationManager

enum TunnelEvent {
    case statusUpdate(TunStatus)
}

class PacketTunnelProvider: NEPacketTunnelProvider {
    private static var defaultPathObserverContext = 0

    private lazy var logger = Logger(label: "MixnetTunnel")

    private let eventStream: AsyncStream<TunnelEvent>
    private let eventContinuation: AsyncStream<TunnelEvent>.Continuation

    private let stateLock = NSLock()
    private var defaultPathObserver: (any OsDefaultPathObserver)?
    private var installedDefaultPathObserver = false

    override init() {
        LoggingSystem.bootstrap { label in
            let fileLogHandler = FileLogHandler(label: label)

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

        let (eventStream, eventContinuation) = AsyncStream<TunnelEvent>.makeStream()
        self.eventStream = eventStream
        self.eventContinuation = eventContinuation
    }

    deinit {
        removeDefaultPathObserver()
    }

    override func startTunnel(options: [String: NSObject]? = nil) async throws {
        logger.info("Start tunnel...")

        setup()

        guard let tunnelProviderProtocol = protocolConfiguration as? NETunnelProviderProtocol,
              let mixnetConfig = tunnelProviderProtocol.asMixnetConfig() else {
            logger.error("Failed to obtain tunnel configuration")
            throw PacketTunnelProviderError.invalidSavedConfiguration
        }

        var vpnConfig = try mixnetConfig.asVpnConfig(tunProvider: self, tunStatusListener: self)
        // todo: figure out WHAT always defaults this setting to .randomLowLatency
        vpnConfig.entryGateway = .random

        logger.info("Starting backend")
        do {
            try startVpn(config: vpnConfig)
        } catch {
            throw PacketTunnelProviderError.backendStartFailure
        }
        logger.info("Backend is up an running...")

        for await event in eventStream {
            switch event {
            case .statusUpdate(.up):
                self.logger.debug("Tunnel is up.")
                // Stop blocking startTunnel() to avoid being terminated due to system 60s timeout
                return

            case .statusUpdate(.establishingConnection):
                self.logger.debug("Tunnel is establishing connection.")

            case .statusUpdate(.down):
                throw PacketTunnelProviderError.backendStartFailure

            case .statusUpdate(.initializingClient):
                self.logger.debug("Initializing the client")

            case .statusUpdate(.disconnecting):
                self.logger.debug("Disconnecting?")
            }
        }
    }

    override func stopTunnel(with reason: NEProviderStopReason) async {
        logger.info("Stop tunnel...")

        do {
            try stopVpn()
        } catch let error {
            logger.error("Failed to stop the tunnel: \(error)")
        }
    }
}

extension PacketTunnelProvider {

    func setup() {
        do {
            try ConfigurationManager.configureCanaryEnvironmentVariables()
        } catch {
            self.logger.error("Failed to set environment: \(error)")
        }
        addDefaultPathObserver()
    }

    func addDefaultPathObserver() {
        guard !installedDefaultPathObserver else { return }
        installedDefaultPathObserver = true
        self.addObserver(self, forKeyPath: #keyPath(defaultPath), context: &Self.defaultPathObserverContext)
    }

    func removeDefaultPathObserver() {
        guard installedDefaultPathObserver else { return }
        installedDefaultPathObserver = false
        self.removeObserver(self, forKeyPath: #keyPath(defaultPath), context: &Self.defaultPathObserverContext)
    }

    func notifyDefaultPathObserver() {
        guard let defaultPath else { return }

        let observer = stateLock.withLock { defaultPathObserver }
        observer?.onDefaultPathChange(newPath: defaultPath.asOsDefaultPath())
    }

    // swiftlint:disable:next block_based_kvo
    override func observeValue(
        forKeyPath keyPath: String?,
        of object: Any?,
        change: [NSKeyValueChangeKey: Any]?,
        context: UnsafeMutableRawPointer?
    ) {
        if keyPath == #keyPath(defaultPath) && context == &Self.defaultPathObserverContext {
            notifyDefaultPathObserver()
        } else {
            super.observeValue(forKeyPath: keyPath, of: object, change: change, context: context)
        }
    }
}

extension PacketTunnelProvider: TunnelStatusListener {
    func onBandwidthStatusChange(status: BandwidthStatus) {
        // todo: implement
    }

    func onConnectionStatusChange(status: ConnectionStatus) {
        // todo: implement
    }

    func onNymVpnStatusChange(status: NymVpnStatus) {
        // todo: implement
    }

    func onExitStatusChange(status: ExitStatus) {
        // todo: implement
    }

    func onTunStatusChange(status: TunStatus) {
        eventContinuation.yield(.statusUpdate(status))
    }
}

extension PacketTunnelProvider: OsTunProvider {
    func setDefaultPathObserver(observer: (any OsDefaultPathObserver)?) throws {
        stateLock.withLock {
            defaultPathObserver = observer
        }
    }

    func setTunnelNetworkSettings(tunnelSettings: TunnelNetworkSettings) async throws {
        do {
            let networkSettings = tunnelSettings.asPacketTunnelNetworkSettings()

            self.logger.debug("Set network settings: \(networkSettings)")

            try await setTunnelNetworkSettings(networkSettings)
        } catch {
            self.logger.error("Failed to set tunnel network settings: \(error)")

            throw error
        }
    }
}
