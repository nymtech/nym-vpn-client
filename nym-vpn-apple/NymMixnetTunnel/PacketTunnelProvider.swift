import NetworkExtension
import Logging
import ConfigurationManager
import NymLogger
import MixnetLibrary
import TunnelMixnet
import Tunnels

class PacketTunnelProvider: NEPacketTunnelProvider {
    private let secondInNanoseconds: UInt64 = 1000000000
    private let eventStream: AsyncStream<TunnelEvent>
    private let stateLock = NSLock()

    let eventContinuation: AsyncStream<TunnelEvent>.Continuation

    private static var defaultPathObserverContext = 0

    lazy var logger = Logger(label: "MixnetTunnel")

    private var defaultPathObserver: (any OsDefaultPathObserver)?
    private var installedDefaultPathObserver = false

    var connectionStartDate: Date?
    var didSendError = false
    var lastErrorStateReason: ErrorStateReason?

    override init() {
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
              let mixnetConfig = tunnelProviderProtocol.asMixnetConfig()
        else {
            logger.error("Failed to obtain tunnel configuration")
            throw PacketTunnelProviderError.invalidSavedConfiguration
        }

        let vpnConfig = try mixnetConfig.asVpnConfig(tunProvider: self, tunStatusListener: self)

        logger.info("Starting backend")
        do {
            try startVpn(config: vpnConfig)
        } catch {
            throw PacketTunnelProviderError.backendStartFailure
        }
        logger.info("Backend is up and running...")
        connectionStartDate = Date()

        // Monitor for didSendError changes concurrently
        try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<Void, Error>) in
            Task {
                await monitorDidSendError { errorOccurred in
                    if errorOccurred {
                        continuation.resume(throwing: PacketTunnelProviderError.backendStartFailure)
                    } else {
                        continuation.resume(returning: ())
                    }
                }
            }

            Task {
                for await event in eventStream {
                    switch event {
                    case .statusUpdate(.up):
                        logger.debug("Tunnel is up.")
                        continuation.resume(returning: ())
                        return
                    case .statusUpdate(.establishingConnection):
                        logger.debug("Tunnel is establishing connection.")
                    case .statusUpdate(.down):
                        logger.error("Failed to start backend")
                        if didSendError {
                            continuation.resume(throwing: PacketTunnelProviderError.backendStartFailure)
                            break
                        }
                    case .statusUpdate(.initializingClient):
                        logger.debug("Initializing the client")
                    case .statusUpdate(.disconnecting):
                        logger.debug("Disconnecting?")
                    }
                }
            }
        }
    }

    override func stopTunnel(with reason: NEProviderStopReason) async {
        logger.info("Stop tunnel... \(reason.rawValue)")

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
            try ConfigurationManager.shared.setup()
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

extension PacketTunnelProvider: OsTunProvider {
    func setDefaultPathObserver(observer: (any OsDefaultPathObserver)?) throws {
        stateLock.withLock {
            defaultPathObserver = observer
        }
    }

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

private extension PacketTunnelProvider {
    func monitorDidSendError(onError: @escaping (Bool) -> Void) async {
        while true {
            logger.info("monitorDidSendError: \(didSendError)")
            if didSendError {
                onError(true)
                break
            }
            try? await Task.sleep(nanoseconds: secondInNanoseconds)
        }
    }
}
