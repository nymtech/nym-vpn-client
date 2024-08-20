import NetworkExtension
import Logging
import NymLogger
import MixnetLibrary
import TunnelMixnet
import Tunnels
import Constants

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
                let osLogHandler = OSLogHandler(subsystem: Bundle.main.bundleIdentifier!, category: label);
                return MultiplexLogHandler([osLogHandler, fileLogHandler])
            #else
                return fileLogHandler
            #endif
        }

        let (eventStream, eventContinuation) = AsyncStream<TunnelEvent>.makeStream();
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
        setupEnvironmentVariables()
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

    override func observeValue(forKeyPath keyPath: String?, of object: Any?, change: [NSKeyValueChangeKey : Any]?, context: UnsafeMutableRawPointer?) {
        if keyPath == #keyPath(defaultPath) && context == &Self.defaultPathObserverContext {
            notifyDefaultPathObserver()
        } else {
            super.observeValue(forKeyPath: keyPath, of: object, change: change, context: context)
        }
    }

    func setupEnvironmentVariables() {
        setenv("CONFIGURED", "true", 1)
        setenv("RUST_LOG", "debug", 1)
        setenv("RUST_BACKTRACE", "1", 1)
        setenv("NETWORK_NAME", "mainnet", 1)
        setenv("BECH32_PREFIX", "n", 1)
        setenv("MIX_DENOM", "unym", 1)
        setenv("MIX_DENOM_DISPLAY", "nym", 1)
        setenv("STAKE_DENOM", "unyx", 1)
        setenv("STAKE_DENOM_DISPLAY", "nyx", 1)
        setenv("DENOMS_EXPONENT", "6", 1)

        setenv("REWARDING_VALIDATOR_ADDRESS", "n10yyd98e2tuwu0f7ypz9dy3hhjw7v772q6287gy", 1)
        setenv("MIXNET_CONTRACT_ADDRESS", "n17srjznxl9dvzdkpwpw24gg668wc73val88a6m5ajg6ankwvz9wtst0cznr", 1)
        setenv("VESTING_CONTRACT_ADDRESS", "n1nc5tatafv6eyq7llkr2gv50ff9e22mnf70qgjlv737ktmt4eswrq73f2nw", 1)

        setenv("STATISTICS_SERVICE_DOMAIN_ADDRESS", "https://mainnet-stats.nymte.ch:8090", 1)
        setenv("NYXD", "https://rpc.nymtech.net", 1)
        setenv("NYXD_WS", "wss://rpc.nymtech.net/websocket", 1)
        setenv("NYM_API", Constants.apiUrl.rawValue, 1)
        setenv("NYM_VPN_API", Constants.nymVpnApi.rawValue, 1)
    }

    func setupSandboxEnvironmentVariables() {
        setenv("CONFIGURED", "true", 1)
        setenv("RUST_LOG", "info", 1)
        setenv("RUST_BACKTRACE", "1", 1)
        setenv("NETWORK_NAME", "sandbox", 1)
        setenv("BECH32_PREFIX", "n", 1)
        setenv("MIX_DENOM", "unym", 1)
        setenv("MIX_DENOM_DISPLAY", "nym", 1)
        setenv("STAKE_DENOM", "unyx", 1)
        setenv("STAKE_DENOM_DISPLAY", "nyx", 1)
        setenv("DENOMS_EXPONENT", "6", 1)

        setenv("REWARDING_VALIDATOR_ADDRESS", "n1duuyj2th2y0z4u4f4wtljpdz9s3pxtu0xx6zdz", 1)
        setenv("MIXNET_CONTRACT_ADDRESS", "n14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9sjyvg3g", 1)
        setenv("COCONUT_BANDWIDTH_CONTRACT_ADDRESS", "n1mf6ptkssddfmxvhdx0ech0k03ktp6kf9yk59renau2gvht3nq2gqt5tdrk", 1)
        setenv("GROUP_CONTRACT_ADDRESS", "n1qg5ega6dykkxc307y25pecuufrjkxkaggkkxh7nad0vhyhtuhw3sa07c47", 1)
        setenv("MULTISIG_CONTRACT_ADDRESS", "n1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqx5a364", 1)
        setenv("COCONUT_DKG_CONTRACT_ADDRESS", "n1aakfpghcanxtc45gpqlx8j3rq0zcpyf49qmhm9mdjrfx036h4z5sy2vfh9", 1)

        setenv("EXPLORER_API", Constants.sandboxExplorerURL.rawValue, 1)
        setenv("NYXD", "https://canary-validator.performance.nymte.ch", 1)
        setenv("NYM_API", Constants.sandboxApiUrl.rawValue, 1)
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

extension TunnelNetworkSettings {
    func asPacketTunnelNetworkSettings() -> NEPacketTunnelNetworkSettings {
        let networkSettings = NEPacketTunnelNetworkSettings(tunnelRemoteAddress: tunnelRemoteAddress)
        networkSettings.ipv4Settings = ipv4Settings?.asNEIPv4Settings()
        networkSettings.ipv6Settings = ipv6Settings?.asNEIPv6Settings()
        networkSettings.mtu = NSNumber(value: mtu)
        networkSettings.dnsSettings = dnsSettings?.asNEDNSSettings()

        return networkSettings
    }
}

extension DnsSettings {
    func asNEDNSSettings() -> NEDNSSettings {
        let dnsSettings = NEDNSSettings(servers: servers)
        dnsSettings.searchDomains = searchDomains
        dnsSettings.matchDomains = matchDomains
        return dnsSettings
    }
}

extension Ipv4Settings {
    func asNEIPv4Settings() -> NEIPv4Settings {
        var addresses = [String]()
        var netmasks = [String]()

        for address in self.addresses {
            if let addrRange = IPAddressRange(from: address) {
                if let ipv4Addr = addrRange.address as? IPv4Address {
                    addresses.append("\(ipv4Addr)")
                    netmasks.append("\(addrRange.subnetMask())")
                }
            }
        }

        let ipv4Settings = NEIPv4Settings(addresses: addresses, subnetMasks: netmasks)
        ipv4Settings.includedRoutes = includedRoutes?.map { $0.asNEIPv4Route() }
        ipv4Settings.excludedRoutes = excludedRoutes?.map { $0.asNEIPv4Route() }

        return ipv4Settings
    }
}

extension Ipv6Settings {
    func asNEIPv6Settings() -> NEIPv6Settings {
        var addresses = [String]()
        var networkPrefixes = [NSNumber]()

        for address in self.addresses {
            if let addrRange = IPAddressRange(from: address) {
                if let ipv6Addr = addrRange.address as? IPv6Address {
                    addresses.append("\(ipv6Addr)")
                    networkPrefixes.append(NSNumber(value: addrRange.networkPrefixLength))
                }
            }
        }

        let ipv6Settings = NEIPv6Settings(addresses: addresses, networkPrefixLengths: networkPrefixes)
        ipv6Settings.includedRoutes = includedRoutes?.map { $0.asNEIPv6Route() }
        ipv6Settings.excludedRoutes = excludedRoutes?.map { $0.asNEIPv6Route() }
        return ipv6Settings
    }
}


extension Ipv4Route {
    func asNEIPv4Route() -> NEIPv4Route {
        switch self {
        case .default:
            return NEIPv4Route.default()

        case let .specific(destination, subnetMask, gateway):
            let ipv4Route = NEIPv4Route(destinationAddress: destination, subnetMask: subnetMask)
            ipv4Route.gatewayAddress = gateway
            return ipv4Route
        }
    }
}

extension Ipv6Route {
    func asNEIPv6Route() -> NEIPv6Route {
        switch self {
        case .default:
            return NEIPv6Route.default()

        case let .specific(destination, prefixLength, gateway):
            let ipv6Route = NEIPv6Route(destinationAddress: destination, networkPrefixLength: NSNumber(value: prefixLength))
            ipv6Route.gatewayAddress = gateway
            return ipv6Route
        }
    }
}

extension NWPath {
    func asOsDefaultPath() -> OsDefaultPath {
        return OsDefaultPath(
            status: status.asOsPathStatus(),
            isExpensive: isExpensive,
            isConstrained: isConstrained
        )
    }
}

extension NWPathStatus {
    func asOsPathStatus() -> OsPathStatus {
        switch self {
        case .invalid: 
            return .invalid
        case .satisfiable:
            return .satisfiable
        case .satisfied:
            return .satisfied
        case .unsatisfied:
            return .unsatisfied
        @unknown default:
            return .unknown(rawValue)
        }
    }
}
