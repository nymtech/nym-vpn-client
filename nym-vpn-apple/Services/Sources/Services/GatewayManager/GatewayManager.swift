import Combine
import Foundation
import AppSettings
import ConfigurationManager
import ConnectionTypes
import Logging
import TunnelStatus
#if os(iOS)
import NymVPNLib
#elseif os(macOS)
import GRPCManager
#endif

@MainActor public final class GatewayManager: ObservableObject {
    let appSettings: AppSettings
    let configurationManager: ConfigurationManager
#if os(macOS)
    let grpcManager: GRPCManager
#endif
    let worker: GatewayWorker
    let logger = Logger(label: "GatewayManager")

    var isLoading = false
    var gatewayStore = GatewayNodeStore()
    var cancellables = Set<AnyCancellable>()

    private var autoUpdateTask: Task<Void, Never>?
    private var gatewayUpdateTask: Task<Void, Never>?

    @Published public var entry: [GatewayNode]
    @Published public var exit: [GatewayNode]
    @Published public var vpn: [GatewayNode]
    @Published public var entryCountries: [NymCountry]
    @Published public var exitCountries: [NymCountry]
    @Published public var vpnCountries: [NymCountry]
    @Published public private(set) var entryFavorites: [ServerFavorite] = []
    @Published public private(set) var exitFavorites: [ServerFavorite] = []

    public let countriesSupportingRegions = ["US"]

    lazy var iso8601Flexible: ISO8601DateFormatter = {
        let formatter = ISO8601DateFormatter()
        formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        return formatter
    }()

#if os(iOS)
    public static let shared = GatewayManager(appSettings: .shared, configurationManager: .shared)
    public init(appSettings: AppSettings, configurationManager: ConfigurationManager) {
        self.appSettings = appSettings
        self.configurationManager = configurationManager
        self.entry = []
        self.exit = []
        self.vpn = []
        self.entryCountries = []
        self.exitCountries = []
        self.vpnCountries = []
        self.worker = GatewayWorker(
            appSettings: appSettings,
            configurationManager: configurationManager
        )
        loadGatewayStore()
        loadPrebundledServersIfNecessary()
    }
#elseif os(macOS)
    public static let shared = GatewayManager(
        appSettings: .shared,
        configurationManager: .shared,
        grpcManager: .shared
    )
    public init(
        appSettings: AppSettings,
        configurationManager: ConfigurationManager,
        grpcManager: GRPCManager
    ) {
        self.appSettings = appSettings
        self.configurationManager = configurationManager
        self.grpcManager = grpcManager
        self.entry = []
        self.exit = []
        self.vpn = []
        self.entryCountries = []
        self.exitCountries = []
        self.vpnCountries = []
        self.worker = GatewayWorker(
            appSettings: appSettings,
            configurationManager: configurationManager,
            grpcManager: grpcManager
        )
        loadGatewayStore()
        loadPrebundledServersIfNecessary()
        setupDaemonObserver()
    }
#endif

    public func setup() {
        updateGateways()
        setupAutoUpdates()
        configureEnvironmentChange()
    }

    public func moniker(with gatewayId: String?) -> String? {
        entry.first(where: { $0.id == gatewayId })?.name
        ?? exit.first(where: { $0.id == gatewayId })?.name
        ?? vpn.first(where: { $0.id == gatewayId })?.name
    }

    public func country(with code: String, gatewayType: NodeType) -> NymCountry? {
        let gateway: GatewayNode?
        switch gatewayType {
        case .entry:
            gateway = entry.first(where: { $0.location?.twoLetterIsoCountryCode == code })
        case .exit:
            gateway = exit.first(where: { $0.location?.twoLetterIsoCountryCode == code })
        case .vpn:
            gateway = vpn.first(where: { $0.location?.twoLetterIsoCountryCode == code })
        }
        if let gateway {
            return localizedCountry(with: gateway.location?.twoLetterIsoCountryCode)
        } else {
            return nil
        }
    }

    public func country(with gatewayId: String?, nodeType: NodeType) -> NymCountry? {
        guard let gatewayId else { return nil }
        switch nodeType {
        case .entry:
            let code = entry.first { $0.id == gatewayId }?.location?.twoLetterIsoCountryCode
            return localizedCountry(with: code)
        case .exit:
            let code = exit.first { $0.id == gatewayId }?.location?.twoLetterIsoCountryCode
            return localizedCountry(with: code)
        case .vpn:
            let code = vpn.first { $0.id == gatewayId }?.location?.twoLetterIsoCountryCode
            return localizedCountry(with: code)
        }
    }

    public func countryCode(with gateway: EntryGateway) -> String? {
        switch gateway {
        case let .country(code):
            return code
        case let .region(countryCode: code, region: _):
            return localizedCountry(with: code)?.code
        case let .gateway(identifier):
            return country(with: identifier, nodeType: .entry)?.code
            ?? country(with: identifier, nodeType: .vpn)?.code
        case .random, .auto:
            return nil
        }
    }

    public func countryCode(with router: ExitRouter) -> String? {
        switch router {
        case let .country(code):
            return code
        case let .gateway(identifier):
            return country(with: identifier, nodeType: .exit)?.code
            ?? country(with: identifier, nodeType: .vpn)?.code
        case let .region(countryCode: code, region: _):
            return localizedCountry(with: code)?.code
        case .random, .auto:
            return nil
        }
    }

    public func userFriendlyTitle(with gateway: EntryGateway) -> String? {
        switch gateway {
        case let .country(code):
            return localizedCountry(with: code)?.name
        case let .region(countryCode: code, region: region):
            if let country = localizedCountry(with: code) {
                return "\(country.name), \(region)"
            } else {
                return region
            }
        case let .gateway(identifier):
            return moniker(with: identifier) ?? identifier
        case .random, .auto:
            return nil
        }
    }

    public func userFriendlyTitle(with router: ExitRouter) -> String? {
        switch router {
        case let .country(code):
            return localizedCountry(with: code)?.name
        case let .gateway(identifier):
            return moniker(with: identifier) ?? identifier
        case let .region(countryCode: code, region: region):
            if let country = localizedCountry(with: code) {
                return "\(country.name), \(region)"
            } else {
                return region
            }
        case .random, .auto:
            return nil
        }
    }

    public func containsQuic(with gateway: EntryGateway) -> Bool {
        switch gateway {
        case let .country(countryCode):
            return vpn.contains { $0.location?.twoLetterIsoCountryCode == countryCode && $0.isQuicAvailable }
        case let .region(countryCode, region):
            return vpn.contains { $0.location?.twoLetterIsoCountryCode == countryCode && $0.location?.region == region }
        case let .gateway(identifier):
            return vpn.contains { $0.id == identifier && $0.isQuicAvailable }
        case .random, .auto:
            return false
        }
    }

    public func containsStreaming(with gateway: ExitRouter) -> Bool {
        switch gateway {
        case .country, .region, .random, .auto:
            false
        case let .gateway(identifier):
            vpn.contains { $0.id == identifier && $0.isResidentialAvailable }
        }
    }
}

// MARK: - Recents -
extension GatewayManager {
    /// Gateways recently connected through, most recent first, for the given tunnel type.
    /// Entry and exit are tracked separately by core.
    public func recentGateways(
        for tunnelType: ConnectionTunnelType
    ) async -> (entry: [GatewayNode], exit: [GatewayNode]) {
        do {
            return try await worker.fetchRecents(for: tunnelType)
        } catch {
            logger.error("Failed to fetch recent gateways: \(error.localizedDescription)")
            return ([], [])
        }
    }
}

// MARK: - Favorites -
extension GatewayManager {
    /// Reload favorites from core. Cheap — reads one small JSON file.
    public func updateFavorites() async {
        do {
            let favorites = try await worker.fetchFavorites()
            // Assign only on change — every publish re-renders the whole gateways screen.
            if entryFavorites != favorites.entry {
                entryFavorites = favorites.entry
            }
            if exitFavorites != favorites.exit {
                exitFavorites = favorites.exit
            }
        } catch {
            logger.error("Failed to fetch favorites: \(error.localizedDescription)")
        }
    }

    public func setEntryFavorite(_ favorite: ServerFavorite, isFavorite: Bool) async {
        do {
            try await worker.setEntryFavorite(favorite, isFavorite: isFavorite)
            await updateFavorites()
        } catch {
            logger.error("Failed to store entry favorite: \(error.localizedDescription)")
        }
    }

    public func setExitFavorite(_ favorite: ServerFavorite, isFavorite: Bool) async {
        do {
            try await worker.setExitFavorite(favorite, isFavorite: isFavorite)
            await updateFavorites()
        } catch {
            logger.error("Failed to store exit favorite: \(error.localizedDescription)")
        }
    }
}

// MARK: - Country -
extension GatewayManager {
    public func shouldDisplayRegion(with countryCode: String) -> Bool {
        countriesSupportingRegions.contains(countryCode)
    }

    public func localizedCountry(with countryCode: String?) -> NymCountry? {
        guard let countryCode,
              !countryCode.isEmpty,
              let countryName = Locale.current.localizedString(forRegionCode: countryCode)
        else {
            return nil
        }
        return NymCountry(name: countryName, code: countryCode, regions: [])
    }
}

// MARK: - Gateway -
extension GatewayManager {
    public func gateway(with gatewayId: String?, gatewayType: NodeType) -> GatewayNode? {
        switch gatewayType {
        case .entry:
            return entry.first(where: { $0.id == gatewayId })
        case .exit:
            return exit.first(where: { $0.id == gatewayId })
        case .vpn:
            return vpn.first(where: { $0.id == gatewayId })
        }
    }

    public func gateways(matching router: ExitRouter, gatewayType: NodeType) -> [GatewayNode] {
        let pool = pool(for: gatewayType)
        switch router {
        case let .country(code):
            return pool.filter { $0.location?.twoLetterIsoCountryCode == code }
        case let .region(countryCode, region):
            return pool.filter {
                $0.location?.twoLetterIsoCountryCode == countryCode && $0.location?.region == region
            }
        case let .gateway(identifier):
            return pool.filter { $0.id == identifier }
        case .random, .auto:
            return pool
        }
    }

    public func gateways(matching gateway: EntryGateway, gatewayType: NodeType) -> [GatewayNode] {
        let pool = pool(for: gatewayType)
        switch gateway {
        case let .country(code):
            return pool.filter { $0.location?.twoLetterIsoCountryCode == code }
        case let .region(countryCode, region):
            return pool.filter {
                $0.location?.twoLetterIsoCountryCode == countryCode && $0.location?.region == region
            }
        case let .gateway(identifier):
            return pool.filter { $0.id == identifier }
        case .random, .auto:
            return pool
        }
    }

    public func bestGateway(matching router: ExitRouter, gatewayType: NodeType) -> GatewayNode? {
        bestScored(in: gateways(matching: router, gatewayType: gatewayType))
    }

    public func bestGateway(matching gateway: EntryGateway, gatewayType: NodeType) -> GatewayNode? {
        bestScored(in: gateways(matching: gateway, gatewayType: gatewayType))
    }

    private func pool(for nodeType: NodeType) -> [GatewayNode] {
        switch nodeType {
        case .entry:
            return entry
        case .exit:
            return exit
        case .vpn:
            return vpn
        }
    }

    private func bestScored(in nodes: [GatewayNode]) -> GatewayNode? {
        nodes.min { $0.mixnetScore.rawValue < $1.mixnetScore.rawValue }
    }
}

// MARK: - Updating countries -
extension GatewayManager {
    func updateGateways() {
        guard !isLoading, needsReload()
        else {
            if entry.isEmpty || exit.isEmpty || vpn.isEmpty {
                loadGatewaysFromStore()
            }
            return
        }
        isLoading = true
        gatewayUpdateTask?.cancel()
        gatewayUpdateTask = Task { [weak self] in
            guard let self else { return }
            await self.fetchGateways()
        }
    }

    func fetchGateways() async {
        defer { isLoading = false }
        do {
            let result = try await worker.fetchGateways()
            logger.info(
                "Fetched gateways entry=\(result.entry.count) exit=\(result.exit.count) vpn=\(result.vpn.count)"
            )

            guard !result.entry.isEmpty || !result.exit.isEmpty || !result.vpn.isEmpty
            else {
                logger.info("Empty gateways from API")
                return
            }

            try Task.checkCancellation()

            entry = result.entry
            exit = result.exit
            vpn = result.vpn

            gatewayStore.entry = result.entry
            gatewayStore.exit  = result.exit
            gatewayStore.vpn = result.vpn
            gatewayStore.lastFetchDate = Date()
#if SANTA
            gatewayStore.fetchedForEnv = configurationManager.currentEnvString
#endif

            storeGatewayStore()
            updateCountriesFromGateways()
        } catch is CancellationError {
            return
        } catch {
            logger.error("Failed to fetch gateways: \(String(describing: error.localizedDescription))")
        }
    }

    func updateCountriesFromGateways() {
        Task {
            let entryRaw = await worker.countries(from: entry)
            let exitRaw = await worker.countries(from: exit)
            let vpnRaw = await worker.countries(from: vpn)

            let localizedEntry = localizeAndSortCountries(entryRaw)
            let localizedExit = localizeAndSortCountries(exitRaw)
            let localizedVpn = localizeAndSortCountries(vpnRaw)

            await MainActor.run {
                entryCountries = localizedEntry
                exitCountries = localizedExit
                vpnCountries = localizedVpn
            }
        }
    }
}

// MARK: - Private helpers (MainActor)
private extension GatewayManager {
    func setupAutoUpdates() {
        autoUpdateTask?.cancel()
        autoUpdateTask = Task { [weak self] in
            guard let self else { return }
            while !Task.isCancelled {
                updateGateways()
                try? await Task.sleep(for: .seconds(300))
            }
        }
    }

    func needsReload() -> Bool {
#if SANTA
        GatewayCacheReloadPolicy.needsReload(
            store: gatewayStore,
            currentEnv: configurationManager.currentEnvString
        )
#else
        guard let lastFetchDate = gatewayStore.lastFetchDate else { return true }
        return Date().timeIntervalSince(lastFetchDate) > 600
#endif
    }

    func loadGatewaysFromStore() {
        exit = gatewayStore.exit
        entry = gatewayStore.entry
        vpn = gatewayStore.vpn
    }

    func configureEnvironmentChange() {
        configurationManager.addEnvironmentDidChangeObserver { [weak self] in
            guard let self else { return }
#if SANTA
            self.clearGatewayStoreForEnvironmentChange()
#else
            self.gatewayStore.lastFetchDate = nil
#endif
            Task { @MainActor in
                self.gatewayUpdateTask?.cancel()
                self.isLoading = false
#if os(iOS)
                await self.worker.reset()
#endif
                self.isLoading = true
                await self.fetchGateways()
            }
        }
    }

    func localizeAndSortCountries(_ countries: [NymCountry]) -> [NymCountry] {
        var localized = countries.compactMap { country -> NymCountry? in
            guard var localizedCountry = localizedCountry(with: country.code) else { return nil }
            localizedCountry.regions = country.regions
            return localizedCountry
        }
        localized.sort { $0.name.localizedCaseInsensitiveCompare($1.name) == .orderedAscending }
        return localized
    }
}
