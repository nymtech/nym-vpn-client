import SwiftUI
import AppSettings
import Constants
import MixnetLibrary

public final class CountriesManager: ObservableObject {
    private var appSettings: AppSettings
    private var isLoading = false
    private var lastHopStore = LastHopStore(lastFetchDate: Date())
    private var entryLastHopStore = EntryLastHopStore(lastFetchDate: Date())

    public static let shared = CountriesManager(appSettings: AppSettings.shared)

    @Published public var entryCountries: [Country]?
    @Published public var exitCountries: [Country]?
    @Published public var lowLatencyCountry: Country?
    @Published public var hasCountries = false

    public init(appSettings: AppSettings) {
        self.appSettings = appSettings
    }

    public func fetchCountries() throws {
        guard !isLoading, needReload(shouldFetchEntryCountries: appSettings.isEntryLocationSelectionOn)
        else {
            loadTemporaryCountries(shouldFetchEntryCountries: appSettings.isEntryLocationSelectionOn)
            return
        }
        isLoading = true

        Task {
            if appSettings.isEntryLocationSelectionOn {
                try fetchEntryExitCountries()
                fetchLowLatencyEntryCountry()
            } else {
                try fetchExitCountries()
            }
        }
    }

    public func country(with code: String, isEntryHop: Bool) -> Country? {
        if isEntryHop {
            return entryCountries?.first(where: { $0.code == code })
        } else {
            return exitCountries?.first(where: { $0.code == code })
        }
    }
}

// MARK: - Fetching -
private extension CountriesManager {
    func fetchEntryExitCountries() throws {
        guard
            let apiURL = URL(string: Constants.apiUrl.rawValue),
            let explorerURL = URL(string: Constants.explorerURL.rawValue),
            let harbourURL = URL(string: Constants.harbourURL.rawValue)
        else {
            throw GeneralNymError.invalidUrl
        }

        let locations = try getGatewayCountries(
            apiUrl: apiURL,
            explorerUrl: explorerURL,
            harbourMasterUrl: harbourURL,
            exitOnly: false
        )
        let newEntryCountries = convertToCountriesAndSort(from: locations)
        let newExitCountries = convertToCountriesAndSort(from: locations)

        entryLastHopStore.entryCountries = newEntryCountries
        entryLastHopStore.exitCountries = newExitCountries
        entryLastHopStore.lastFetchDate = Date()

        entryCountries = newEntryCountries
        exitCountries = newExitCountries
        isLoading = false
    }

    func fetchExitCountries() throws {
        guard
            let apiURL = URL(string: Constants.apiUrl.rawValue),
            let explorerURL = URL(string: Constants.explorerURL.rawValue),
            let harbourURL = URL(string: Constants.harbourURL.rawValue)
        else {
            throw GeneralNymError.invalidUrl
        }

        let locations = try getGatewayCountries(
            apiUrl: apiURL,
            explorerUrl: explorerURL,
            harbourMasterUrl: harbourURL,
            exitOnly: true
        )
        let newExitCountries = convertToCountriesAndSort(from: locations)

        lastHopStore.countries = newExitCountries
        lastHopStore.lastFetchDate = Date()

        entryCountries = nil
        exitCountries = newExitCountries
        isLoading = false
        updateHasCountries()
    }

    func fetchLowLatencyEntryCountry() {
        guard
            let apiURL = URL(string: Constants.apiUrl.rawValue),
            let explorerURL = URL(string: Constants.explorerURL.rawValue),
            let harbourURL = URL(string: Constants.harbourURL.rawValue),
            let location = try? getLowLatencyEntryCountry(
                apiUrl: apiURL,
                explorerUrl: explorerURL,
                harbourMasterUrl: harbourURL
            )
        else {
            return
        }
        entryLastHopStore.lowLatencyCountry = lowLatencyCountry
        lastHopStore.lowLatencyCountry = lowLatencyCountry
        lowLatencyCountry = Country(name: location.countryName, code: location.twoLetterIsoCountryCode)
        updateHasCountries()
    }
}

// MARK: - Temp storage -
private extension CountriesManager {
    func needReload(shouldFetchEntryCountries: Bool) -> Bool {
        if shouldFetchEntryCountries {
            guard let countries = entryLastHopStore.entryCountries, !countries.isEmpty else { return true }
        } else {
            guard let countries = lastHopStore.countries, !countries.isEmpty else { return true }
        }

        if shouldFetchEntryCountries {
            let lastFetchDate = entryLastHopStore.lastFetchDate
            return isLongerThan10Minutes(date: lastFetchDate)
        } else {
            let lastFetchDate = lastHopStore.lastFetchDate
            return isLongerThan10Minutes(date: lastFetchDate)
        }
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
            if shouldFetchEntryCountries {
                exitCountries = entryLastHopStore.exitCountries
                entryCountries = entryLastHopStore.entryCountries
                lowLatencyCountry = entryLastHopStore.lowLatencyCountry
            } else {
                exitCountries = lastHopStore.countries
                entryCountries = nil
                lowLatencyCountry = nil
            }
            updateHasCountries()
        }
    }
}

// MARK: - ConvertToCountry -
private extension CountriesManager {
    func convertToCountriesAndSort(from locations: [Location]) -> [Country] {
        locations.map {
            Country(name: $0.countryName, code: $0.twoLetterIsoCountryCode)
        }
        .sorted { $0.name < $1.name }
    }
}

// MARK: - Helper -
private extension CountriesManager {
    func updateHasCountries() {
        if appSettings.isEntryLocationSelectionOn {
            hasCountries = ((entryCountries?.isEmpty) != nil)
        } else {
            hasCountries = ((exitCountries?.isEmpty) != nil)
        }
    }
}
