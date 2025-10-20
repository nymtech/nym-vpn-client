import Combine
import Foundation
import AppSettings
import ConfigurationManager
import ConnectionTypes
import CountriesManagerTypes
import Logging
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

    @Published public var entry: [GatewayNode]
    @Published public var exit: [GatewayNode]
    @Published public var vpn: [GatewayNode]
    @Published public var entryCountries: [NymCountry]
    @Published public var exitCountries: [NymCountry]
    @Published public var vpnCountries: [NymCountry]
    @Published public var lastError: Error?

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

    public func localizedCountry(with countryCode: String?) -> NymCountry? {
        guard let countryCode,
              !countryCode.isEmpty,
              let countryName = Locale.current.localizedString(forRegionCode: countryCode)
        else {
            return nil
        }
        return NymCountry(name: countryName, code: countryCode, regions: [])
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
        case let .country(code), let .lowLatencyCountry(code):
            return code
        case let .region(countryCode: code, region: _):
            return localizedCountry(with: code)?.code
        case .city:
            return nil
        case let .gateway(identifier):
            return country(with: identifier, nodeType: .entry)?.code
            ?? country(with: identifier, nodeType: .vpn)?.code
        case .random:
            return nil
        }
    }

    public func countryCode(with router: ExitRouter) -> String? {
        switch router {
        case .address:
            return nil
        case let .country(code):
            return code
        case let .gateway(identifier):
            return country(with: identifier, nodeType: .exit)?.code
            ?? country(with: identifier, nodeType: .vpn)?.code
        case let .region(countryCode: code, region: _):
            return localizedCountry(with: code)?.code
        case .random:
            return nil
        }
    }

    public func userFriendlyTitle(with gateway: EntryGateway) -> String? {
        switch gateway {
        case let .country(code), let .lowLatencyCountry(code):
            return localizedCountry(with: code)?.name
        case let .region(countryCode: code, region: region):
            if let country = localizedCountry(with: code) {
                return "\(country.name), \(region)"
            } else {
                return region
            }
        case .city:
            return nil
        case let .gateway(identifier):
            return moniker(with: identifier) ?? identifier
        case .random:
            return nil
        }
    }

    public func userFriendlyTitle(with router: ExitRouter) -> String? {
        switch router {
        case .address:
            return nil
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
        case .random:
            return nil
        }
    }
}

// MARK: Updating countries
extension GatewayManager {
    func fetchGateways() async {
        do {
            let result = try await Task.detached { [worker] in
                try await worker.fetchGateways()
            }.value

            guard !result.entry.isEmpty, !result.exit.isEmpty, !result.vpn.isEmpty
            else {
                logger.info("Empty gateways from API")
                isLoading = false
                return
            }

            entry = result.entry
            exit = result.exit
            vpn = result.vpn

            gatewayStore.entry = result.entry
            gatewayStore.exit  = result.exit
            gatewayStore.vpn = result.vpn
            gatewayStore.lastFetchDate = Date()

            storeGatewayStore()
            updateCountriesFromGateways()
            isLoading = false
        } catch {
            logger.error("Failed to fetch gateways: \(String(describing: error))")
            updateError(with: error)
            isLoading = false
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

    private func localizeAndSortCountries(_ countries: [NymCountry]) -> [NymCountry] {
        var localized = countries.compactMap { country -> NymCountry? in
            guard var localizedCountry = localizedCountry(with: country.code) else { return nil }
            localizedCountry.regions = country.regions
            return localizedCountry
        }
        localized.sort { $0.name.localizedCaseInsensitiveCompare($1.name) == .orderedAscending }
        return localized
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

    func updateGateways() {
        guard !isLoading, needsReload()
        else {
            if entry.isEmpty || exit.isEmpty || vpn.isEmpty {
                loadGatewaysFromStore()
            }
            return
        }
        isLoading = true

        Task { [weak self] in
            guard let self else { return }
            await self.fetchGateways()
        }
    }

    func needsReload() -> Bool {
        guard let lastFetchDate = gatewayStore.lastFetchDate else { return true }
        return Date().timeIntervalSince(lastFetchDate) > 600
    }

    func loadGatewaysFromStore() {
        exit = gatewayStore.exit
        entry = gatewayStore.entry
        vpn = gatewayStore.vpn
    }

    func configureEnvironmentChange() {
        configurationManager.environmentDidChange = { [weak self] in
            guard let self else { return }
            self.gatewayStore.lastFetchDate = nil
            Task {
                try? await Task.sleep(for: .seconds(3))
                await self.fetchGateways()
            }
        }
    }
}

extension GatewayManager {
    func updateError(with error: Error) {
        lastError = error
    }
}
