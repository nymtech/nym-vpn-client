import AppSettings
import ConfigurationManager
import ConnectionTypes
import PathManager
import TunnelStatus
#if os(iOS)
import NymVPNLib
#elseif os(macOS)
import GRPCManager
#endif
import Logging

#if os(iOS)
private extension ConnectionTunnelType {
    var libValue: NymVPNLib.TunnelType {
        switch self {
        case .mixnet:
            .mixnet
        case .wireguard:
            .wireguard
        }
    }
}
#endif

#if os(iOS)
enum State {
    /// Actor is initialized but does not hold gateway cache
    case initial

    /// Actor is loading gateway cache
    case loading(Task<NymGatewayCache, Error>)

    /// Actor is ready, gateway cache is initialized
    case ready(NymGatewayCache)
}
#endif

actor GatewayWorker {
    let appSettings: AppSettings
    let configurationManager: ConfigurationManager
    private let logger = Logger(label: "GatewayWorker")
#if os(macOS)
    let grpcManager: GRPCManager
#endif
#if os(iOS)
    var state = State.initial
#endif

#if os(iOS)
    init(appSettings: AppSettings, configurationManager: ConfigurationManager) {
        self.appSettings = appSettings
        self.configurationManager = configurationManager
    }
#elseif os(macOS)
    init(appSettings: AppSettings, configurationManager: ConfigurationManager, grpcManager: GRPCManager) {
        self.appSettings = appSettings
        self.configurationManager = configurationManager
        self.grpcManager = grpcManager
    }
#endif

#if os(iOS)

    /// Reset internal gateway cache in order for it to be re-initialized during the next attempt to fetch gateways
    /// Use that only on environment changes because that wipes out in-memory gateway cache
    func reset() async {
        switch state {
        case .initial:
            break

        case let .loading(task):
            task.cancel()
            state = .initial

        case .ready:
            state = .initial
        }
    }

    func fetchGateways() async throws -> (entry: [GatewayNode], exit: [GatewayNode], vpn: [GatewayNode]) {
        let gatewayCache = try await gatewayCache()

        let entryNodes = try await gatewayCache.getGateways(gwType: .mixnetEntry)
        let exitNodes = try await gatewayCache.getGateways(gwType: .mixnetExit)
        let vpnNodes = try await gatewayCache.getGateways(gwType: .wg)

        let entry = entryNodes.map { GatewayNode(with: $0) }
        let exit = exitNodes.map { GatewayNode(with: $0) }
        let vpn = vpnNodes.map { GatewayNode(with: $0) }

        return (entry, exit, vpn)
    }

    /// Recently connected gateways, read from the network data dir the tunnel extension writes them to.
    /// Runs in the app process — no tunnel and no vpn service required.
    func fetchRecents(
        for tunnelType: ConnectionTunnelType
    ) async throws -> (entry: [GatewayNode], exit: [GatewayNode]) {
        guard let networkName = await configurationManager.networkEnv?.networkName() else { return ([], []) }
        let networkDataURL = try PathManager.dataFolderURL().appendingPathComponent(networkName)
        let recents = try await getRecentGatewaysNoService(
            dataDir: networkDataURL.path(),
            gatewayCache: gatewayCache(),
            tunnelType: tunnelType.libValue
        )
        return (recents.entry.map { GatewayNode(with: $0) }, recents.exit.map { GatewayNode(with: $0) })
    }

    private func gatewayCache() async throws -> NymGatewayCache {
        let gatewayCache: NymGatewayCache

        switch state {
        case .initial:
            let task = Task {
                let networkName = await configurationManager.networkEnv?.networkName()
                logger.info("Setup gateway cache with \(String(describing: networkName))")

                let offlineMonitor = await NymOfflineMonitor()
                let gatewayCache = try await NymGatewayCache(
                    userAgent: .appUserAgent,
                    environment: configurationManager.networkEnv ?? .newWithMainnetFallback(),
                    offlineMonitor: offlineMonitor
                )

                try Task.checkCancellation()

                return gatewayCache
            }

            state = .loading(task)

            do {
                gatewayCache = try await task.value
                state = .ready(gatewayCache)
            } catch {
                state = .initial
                if !(error is CancellationError) {
                    logger.error("Failed to initialize gateway cache: \(error)")
                }
                throw error
            }

        case let .loading(task):
            gatewayCache = try await task.value

        case let .ready(gwCache):
            gatewayCache = gwCache
        }

        return gatewayCache
    }
#elseif os(macOS)
    func fetchGateways() async throws -> (entry: [GatewayNode], exit: [GatewayNode], vpn: [GatewayNode]) {
        let entry = try await grpcManager.gateways(for: .entry)
        let exit = try await grpcManager.gateways(for: .exit)
        let vpn = try await grpcManager.gateways(for: .vpn)
        return (entry, exit, vpn)
    }

    /// Recently connected gateways, as recorded by the daemon.
    func fetchRecents(
        for tunnelType: ConnectionTunnelType
    ) async throws -> (entry: [GatewayNode], exit: [GatewayNode]) {
        try await grpcManager.recentGateways(for: tunnelType)
    }
#endif

    func countries(from nodes: [GatewayNode]) -> [NymCountry] {
        // countryCode → regionName → Set<cityName>
        var citiesByRegionByCountry: [String: [String: Set<String>]] = [:]
        nodes.compactMap(\.location).forEach { location in
            let code = location.twoLetterIsoCountryCode.uppercased()
            let region = location.region.trimmingCharacters(in: .whitespacesAndNewlines)
            let city = location.city.trimmingCharacters(in: .whitespacesAndNewlines)

            guard !region.isEmpty, !city.isEmpty else { return }

            citiesByRegionByCountry[code, default: [:]][region, default: []].insert(city)
        }

        var result: [NymCountry] = []
        result.reserveCapacity(citiesByRegionByCountry.count)

        for (code, regionsDict) in citiesByRegionByCountry {
            let regions: [NymCountry.Region] = regionsDict
                .map { regionName, citiesSet in
                    let cities = citiesSet.sorted {
                        $0.localizedCaseInsensitiveCompare($1) == .orderedAscending
                    }
                    return NymCountry.Region(name: regionName, cities: cities)
                }
                .sorted { $0.name.localizedCaseInsensitiveCompare($1.name) == .orderedAscending }

            result.append(NymCountry(name: code, code: code, regions: regions)) // name localized later
        }
        result.sort { $0.name.localizedCaseInsensitiveCompare($1.name) == .orderedAscending }
        return result
    }
}
