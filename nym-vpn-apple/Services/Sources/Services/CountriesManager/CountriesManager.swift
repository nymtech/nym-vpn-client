import Combine
import SwiftUI
import AppSettings
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
    private var appSettings: AppSettings

    let logger = Logger(label: "CountriesManager")
#if os(macOS)
    let grpcManager: GRPCManager
    let helperManager: HelperManager
#endif

    var isLoading = false
    var timer: Timer?
    var entryLastHopStore = EntryLastHopStore()
    var cancellables = Set<AnyCancellable>()
#if os(iOS)
    public static let shared = CountriesManager(appSettings: AppSettings.shared)
#endif
#if os(macOS)
    public static let shared = CountriesManager(
        appSettings: AppSettings.shared,
        grpcManager: GRPCManager.shared,
        helperManager: HelperManager.shared
    )
#endif

    @Published public var entryCountries: [Country]
    @Published public var exitCountries: [Country]
    @Published public var lastError: Error?

#if os(iOS)
    public init(appSettings: AppSettings) {
        self.appSettings = appSettings
        self.entryCountries = []
        self.exitCountries = []

        setup()
    }
#endif

#if os(macOS)
    public init(appSettings: AppSettings, grpcManager: GRPCManager, helperManager: HelperManager) {
        self.appSettings = appSettings
        self.grpcManager = grpcManager
        self.helperManager = helperManager
        self.entryCountries = []
        self.exitCountries = []

        setup()
    }
#endif

    @objc public func fetchCountries() {
        guard !isLoading, needsReload(shouldFetchEntryCountries: appSettings.isEntryLocationSelectionOn)
        else {
            loadTemporaryCountries(shouldFetchEntryCountries: appSettings.isEntryLocationSelectionOn)
            return
        }
        isLoading = true

        Task {
            fetchEntryExitCountries()
        }
    }

    public func country(with code: String, isEntryHop: Bool) -> Country? {
        if isEntryHop {
            return entryCountries.first(where: { $0.code == code })
        } else {
            return exitCountries.first(where: { $0.code == code })
        }
    }
}

// MARK: - Setup -
private extension CountriesManager {
    func setup() {
        loadPrebundledCountries()
        setupAppSettingsObservers()
        setupAutoUpdates()
        fetchCountries()
    }

    func setupAppSettingsObservers() {
        appSettings.$isEntryLocationSelectionOnPublisher.sink { [weak self] _ in
            self?.fetchCountries()
        }
        .store(in: &cancellables)
    }

    func setupAutoUpdates() {
        timer = Timer.scheduledTimer(
            timeInterval: 600,
            target: self,
            selector: #selector(fetchCountries),
            userInfo: nil,
            repeats: true
        )
    }
}

// MARK: - Pre bundled countries -
private extension CountriesManager {
    func loadPrebundledCountries() {
        guard let entryCountriesURL = Bundle.main.url(forResource: "gatewaysEntryCountries", withExtension: "json"),
              let exitCountriesURL = Bundle.main.url(forResource: "gatewaysExitCountries", withExtension: "json")
        else {
            updateError(with: GeneralNymError.noPrebundledCountries)
            return
        }

        do {
            let prebundledEntryCountries = try loadPrebundledCountries(from: entryCountriesURL)
            let prebundledExitCountries = try loadPrebundledCountries(from: exitCountriesURL)

            entryLastHopStore.entryCountries = prebundledEntryCountries
            entryLastHopStore.exitCountries = prebundledExitCountries

            entryCountries = prebundledEntryCountries
            exitCountries = prebundledExitCountries
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
        guard helperManager.isHelperRunning()
        else {
            fetchCountriesAfterDelay()
            return
        }

        Task {
            do {
                try await fetchEntryCountries()
                try await fetchExitCountries()
            } catch {
                Task { @MainActor in
                    lastError = error
                }
            }
        }
    }

    func fetchEntryCountries() async throws {
            let countryCodes = try await grpcManager.entryCountryCodes()
            let countries = countryCodes.compactMap { countryCode in
                country(with: countryCode)
            }
            .sorted(by: { $0.name < $1.name })

            Task { @MainActor in
                entryLastHopStore.entryCountries = countries
                entryLastHopStore.lastFetchDate = Date()
                entryCountries = countries
            }
    }

    func fetchExitCountries() async throws {
        let countryCodes = try await grpcManager.exitCountryCodes()
        let countries = countryCodes.compactMap { countryCode in
            country(with: countryCode)
        }
        .sorted(by: { $0.name < $1.name })

        Task { @MainActor in
            entryLastHopStore.exitCountries = countries
            entryLastHopStore.lastFetchDate = Date()
            exitCountries = countries
        }
    }
}
#endif

#if os(iOS)
private extension CountriesManager {
    func fetchEntryExitCountries() {
        guard let apiURL = URL(string: Constants.apiUrl.rawValue),
              let explorerURL = URL(string: Constants.explorerURL.rawValue),
              let harbourURL = URL(string: Constants.harbourURL.rawValue)
        else {
            updateError(with: GeneralNymError.cannotFetchCountries)
            return
        }

        do {
            let entryExitLocations = try getGatewayCountries(
                apiUrl: apiURL,
                explorerUrl: explorerURL,
                harbourMasterUrl: harbourURL,
                exitOnly: false
            )
            let newEntryCountries = entryExitLocations.compactMap {
                country(with: $0.twoLetterIsoCountryCode)
            }
            .sorted(by: { $0.name < $1.name })

            let exitLocations = try getGatewayCountries(
                apiUrl: apiURL,
                explorerUrl: explorerURL,
                harbourMasterUrl: harbourURL,
                exitOnly: true
            )
            let newExitCountries = exitLocations.compactMap {
                country(with: $0.twoLetterIsoCountryCode)
            }
                .sorted(by: { $0.name < $1.name })

            entryLastHopStore.entryCountries = entryCountries
            entryLastHopStore.exitCountries = exitCountries
            entryLastHopStore.lastFetchDate = Date()
            entryCountries = newEntryCountries
            exitCountries = newExitCountries

            isLoading = false
        } catch {
            isLoading = false
            updateError(with: error)
            fetchCountriesAfterDelay()
        }
    }
}
#endif

private extension CountriesManager {
    func country(with countryCode: String) -> Country? {
        guard let countryName = Locale.current.localizedString(forRegionCode: countryCode)
        else {
            logger.log(level: .error, "Failed resolving country code for: \(countryCode)")
            return nil
        }
        return Country(name: countryName, code: countryCode)
    }
}

// MARK: - Temp storage -
private extension CountriesManager {
    func needsReload(shouldFetchEntryCountries: Bool) -> Bool {
        guard let lastFetchDate = entryLastHopStore.lastFetchDate else { return true }
        return isLongerThan10Minutes(date: lastFetchDate)
    }

    func isLongerThan10Minutes(date: Date) -> Bool {
        let difference = Date().timeIntervalSince(date)
        if difference > 600 {
            return true
        } else {
            return false
        }
    }

    func loadTemporaryCountries(shouldFetchEntryCountries: Bool) {
        Task { @MainActor in
            exitCountries = entryLastHopStore.exitCountries
            entryCountries = entryLastHopStore.entryCountries
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
        DispatchQueue.main.asyncAfter(deadline: .now() + 15) { [weak self] in
            self?.fetchEntryExitCountries()
        }
    }
}
