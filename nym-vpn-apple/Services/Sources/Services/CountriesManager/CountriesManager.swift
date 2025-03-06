import Combine
import SwiftUI
import AppSettings
import AppVersionProvider
import ConfigurationManager
import CountriesManagerTypes
#if os(macOS)
import GRPCManager
import HelperManager
#endif
#if os(iOS)
import MixnetLibrary
#endif
import Constants
import Logging

public final class CountriesManager: ObservableObject {
    private let appSettings: AppSettings
    private let configurationManager: ConfigurationManager

    let logger = Logger(label: "CountriesManager")
#if os(macOS)
    let grpcManager: GRPCManager
    let helperManager: HelperManager

    var daemonVersion: String?
#endif
#if os(iOS)
    public static let shared = CountriesManager(
        appSettings: AppSettings.shared,
        configurationManager: ConfigurationManager.shared
    )
#endif
#if os(macOS)
    public static let shared = CountriesManager(
        appSettings: .shared,
        grpcManager: .shared,
        helperManager: .shared,
        configurationManager: .shared
    )
#endif
    var isLoading = false
    var timer: Timer?
    var countryStore = CountryStore()
    var cancellables = Set<AnyCancellable>()

    @Published public var entryCountries: [Country]
    @Published public var exitCountries: [Country]
    @Published public var vpnCountries: [Country]
    @Published public var lastError: Error?

#if os(iOS)
    public init(
        appSettings: AppSettings,
        configurationManager: ConfigurationManager
    ) {
        self.appSettings = appSettings
        self.configurationManager = configurationManager
        self.entryCountries = []
        self.exitCountries = []
        self.vpnCountries = []

        loadCountryStore()
        loadPrebundledCountriesIfNecessary()
    }
#elseif os(macOS)
    public init(
        appSettings: AppSettings,
        grpcManager: GRPCManager,
        helperManager: HelperManager,
        configurationManager: ConfigurationManager
    ) {
        self.appSettings = appSettings
        self.configurationManager = configurationManager
        self.grpcManager = grpcManager
        self.helperManager = helperManager
        self.entryCountries = []
        self.exitCountries = []
        self.vpnCountries = []

        loadCountryStore()
        loadPrebundledCountriesIfNecessary()
    }
#endif

    public func setup() {
        setupAutoUpdates()
        configureEnvironmentChange()
        fetchCountries()
#if os(macOS)
        setupDaemonStateObserver()
#endif
    }

    @objc public func fetchCountries() {
        guard !isLoading, needsReload()
        else {
            if entryCountries.isEmpty
                || exitCountries.isEmpty
                || vpnCountries.isEmpty {
                loadCountriesFromCountryStore()
            }
            return
        }
        isLoading = true

        Task { [weak self] in
            self?.fetchEntryExitCountries()
        }
    }

    public func country(with code: String, gatewayType: NodeType) -> Country? {
        switch gatewayType {
        case .entry:
            return entryCountries.first(where: { $0.code == code })
        case .exit:
            return exitCountries.first(where: { $0.code == code })
        case .vpn:
            return vpnCountries.first(where: { $0.code == code })
        }
    }
}

// MARK: - Setup -
private extension CountriesManager {
    func setupAutoUpdates() {
        timer = Timer.scheduledTimer(
            timeInterval: 1800,
            target: self,
            selector: #selector(fetchCountries),
            userInfo: nil,
            repeats: true
        )
    }

    func configureEnvironmentChange() {
        configurationManager.environmentDidChange = { [weak self] in
            self?.countryStore.lastFetchDate = nil
            self?.fetchCountries()
        }
    }

#if os(macOS)
    func setupDaemonStateObserver() {
        helperManager.$daemonState.sink { [weak self] daemonState in
            guard daemonState == .running else { return }
            Task {
                try? await Task.sleep(for: .seconds(5))
                self?.fetchCountries()
            }
        }
        .store(in: &cancellables)
    }
#endif
}

// MARK: - Pre bundled countries -
private extension CountriesManager {
    func loadCountryStore() {
        guard let countryStoreString = appSettings.countryStore,
              let loadedCountryStore = CountryStore(rawValue: countryStoreString)
        else {
            return
        }
        countryStore = loadedCountryStore
        entryCountries = loadedCountryStore.entryCountries
        exitCountries = loadedCountryStore.exitCountries
        vpnCountries = loadedCountryStore.vpnCountries
    }

    func loadPrebundledCountriesIfNecessary() {
        guard entryCountries.isEmpty || exitCountries.isEmpty || vpnCountries.isEmpty else { return }
        guard let entryCountriesURL = Bundle.main.url(forResource: "gatewaysEntryCountries", withExtension: "json"),
              let exitCountriesURL = Bundle.main.url(forResource: "gatewaysExitCountries", withExtension: "json"),
              let vpnCountriesURL = Bundle.main.url(forResource: "gatewaysVpnCountries", withExtension: "json")
        else {
            updateError(with: GeneralNymError.noPrebundledCountries)
            return
        }

        do {
            let prebundledEntryCountries = try loadPrebundledCountries(from: entryCountriesURL)
            let prebundledExitCountries = try loadPrebundledCountries(from: exitCountriesURL)
            let prebundledVPNCountries = try loadPrebundledCountries(from: vpnCountriesURL)

            countryStore.entryCountries = prebundledEntryCountries
            countryStore.exitCountries = prebundledExitCountries
            countryStore.vpnCountries = prebundledVPNCountries

            entryCountries = prebundledEntryCountries
            exitCountries = prebundledExitCountries
            vpnCountries = prebundledVPNCountries

            logger.info("Loading prebundled countries")
            logger.info("entry: \(countryStore.entryCountries.count)")
            logger.info("exit: \(countryStore.exitCountries.count)")
            logger.info("vpn: \(countryStore.vpnCountries.count)")
        } catch let error {
            updateError(with: error)
            return
        }
    }

    func loadPrebundledCountries(from fileURL: URL) throws -> [Country] {
        do {
            let data = try Data(contentsOf: fileURL)
            let countryCodes = try JSONDecoder().decode([String].self, from: data)
            let countries = countryCodes.compactMap { [weak self] countryCode in
                self?.country(with: countryCode)
            }
            .sorted(by: { $0.name < $1.name })

            return countries
        } catch {
            throw GeneralNymError.cannotParseCountries
        }
    }
}

#if os(macOS)
private extension CountriesManager {
    func fetchEntryExitCountries() {
        Task {
            do {
                try await fetchEntryCountries()
                try await fetchExitCountries()
                try await fetchVPNCountries()
            } catch {
                logger.error("\(error.localizedDescription)")
            }
            countryStore.lastFetchDate = Date()
            storeCountryStore()
        }
    }

    func fetchEntryCountries() async throws {
            let countryCodes = try await grpcManager.countryCodes(for: .entry)
            let countries = countryCodes.compactMap { countryCode in
                country(with: countryCode)
            }
            .sorted(by: { $0.name < $1.name })

            logger.info("Fetched \(countries.count) entry countries")

            Task { @MainActor in
                countryStore.entryCountries = countries
                entryCountries = countries
            }
    }

    func fetchExitCountries() async throws {
        let countryCodes = try await grpcManager.countryCodes(for: .exit)
        let countries = countryCodes.compactMap { countryCode in
            country(with: countryCode)
        }
        .sorted(by: { $0.name < $1.name })

        logger.info("Fetched \(countries.count) exit countries")

        Task { @MainActor in
            countryStore.exitCountries = countries
            exitCountries = countries
        }
    }

    func fetchVPNCountries() async throws {
        let countryCodes = try await grpcManager.countryCodes(for: .vpn)
        let countries = countryCodes.compactMap { countryCode in
            country(with: countryCode)
        }
        .sorted(by: { $0.name < $1.name })

        logger.info("Fetched \(countries.count) vpn countries")

        Task { @MainActor in
            countryStore.vpnCountries = countries
            vpnCountries = countries
        }
    }
}
#endif

#if os(iOS)
private extension CountriesManager {
    func fetchEntryExitCountries() {
        do {
            let userAgent = UserAgent(
                application: AppVersionProvider.app,
                version: "\(AppVersionProvider.appVersion()) (\(AppVersionProvider.libVersion))",
                platform: AppVersionProvider.platform,
                gitCommit: ""
            )

            let entryLocations = try getGatewayCountries(
                gwType: .mixnetEntry,
                userAgent: userAgent,
                minGatewayPerformance: nil
            )
            logger.info("Fetched \(entryLocations.count) entry countries")
            let newEntryCountries = entryLocations.compactMap {
                country(with: $0.twoLetterIsoCountryCode)
            }
            .sorted(by: { $0.name < $1.name })

            let exitLocations = try getGatewayCountries(
                gwType: .mixnetExit,
                userAgent: userAgent,
                minGatewayPerformance: nil
            )
            logger.info("Fetched \(exitLocations.count) exit countries")
            let newExitCountries = exitLocations.compactMap {
                country(with: $0.twoLetterIsoCountryCode)
            }
            .sorted(by: { $0.name < $1.name })

            let newVpnLocations = try getGatewayCountries(
                gwType: .wg,
                userAgent: userAgent,
                minGatewayPerformance: nil
            )
            logger.info("Fetched \(newVpnLocations.count) vpn countries")
            let newVpnCountries = newVpnLocations.compactMap {
                country(with: $0.twoLetterIsoCountryCode)
            }
            .sorted(by: { $0.name < $1.name })

            countryStore.entryCountries = newEntryCountries
            countryStore.exitCountries = newExitCountries
            countryStore.vpnCountries = newVpnCountries
            countryStore.lastFetchDate = Date()

            Task { @MainActor [weak self] in
                self?.entryCountries = newEntryCountries
                self?.exitCountries = newExitCountries
                self?.vpnCountries = newVpnCountries
            }

            storeCountryStore()

            isLoading = false
        } catch {
            isLoading = false
            logger.error("\(error.localizedDescription)")
            fetchCountriesAfterDelay()
        }
    }
}
#endif

extension CountriesManager {
    public func country(with countryCode: String) -> Country? {
        guard !countryCode.isEmpty, let countryName = Locale.current.localizedString(forRegionCode: countryCode)
        else {
            logger.log(level: .error, "Failed resolving country code for: \(countryCode)")
            return nil
        }
        return Country(name: countryName, code: countryCode)
    }
}

// MARK: - Temp storage -
private extension CountriesManager {
    func needsReload() -> Bool {
        guard let lastFetchDate = countryStore.lastFetchDate else { return true }
        return isLongerThan30Minutes(date: lastFetchDate)
    }

    func isLongerThan30Minutes(date: Date) -> Bool {
        let difference = Date().timeIntervalSince(date)
        return difference > 1800 ? true : false
    }

    func loadCountriesFromCountryStore() {
        logger.info("Reloading temporary countries")
        Task { @MainActor in
            exitCountries = countryStore.exitCountries
            entryCountries = countryStore.entryCountries
            vpnCountries = countryStore.vpnCountries
        }
    }

    func storeCountryStore() {
        Task { @MainActor in
            appSettings.countryStore = countryStore.rawValue
        }
    }
}

// MARK: - Helper -
extension CountriesManager {
    func updateError(with error: Error) {
        Task { @MainActor in
            lastError = error
        }
    }

    func fetchCountriesAfterDelay() {
        Task { [weak self] in
            try? await Task.sleep(for: .seconds(60))
            self?.fetchEntryExitCountries()
        }
    }
}
