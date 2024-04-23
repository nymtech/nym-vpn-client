import SwiftUI
import Constants
import MixnetLibrary

public final class CountriesManager: ObservableObject {
    private var isLoading = false

    public static let shared = CountriesManager()

    @Published public var entryCountries: [Country]?
    @Published public var exitCountries: [Country]?
    @Published public var lowLatencyCountry: Country?

    public func fetchCountries(shouldFetchEntryCountries: Bool) throws {
        guard !isLoading else { return }
        print("🇬🇧 Fetching countries 🇬🇧")
        isLoading = true

        Task {
            if shouldFetchEntryCountries {
                try fetchEntryCountries()
            } else {
                try fetchExitCountries()
            }
            fetchLowLatencyEntryCountry()
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

private extension CountriesManager {
    func fetchEntryCountries() throws {
        guard
            let apiURL = URL(string: Constants.apiUrl.rawValue),
            let explorerURL = URL(string: Constants.explorerUrl.rawValue)
        else {
            throw GeneralNymError.invalidUrl
        }

        let locations = try getGatewayCountries(
            apiUrl: apiURL,
            explorerUrl: explorerURL,
            exitOnly: false
        )
        let newEntryCountries = convertToCountriesAndSort(from: locations)
        let newExitCountries = convertToCountriesAndSort(from: locations)

        entryCountries = newEntryCountries
        exitCountries = newExitCountries
        isLoading = false
    }

    func fetchExitCountries() throws {
        guard
            let apiURL = URL(string: Constants.apiUrl.rawValue),
            let explorerURL = URL(string: Constants.explorerUrl.rawValue)
        else {
            throw GeneralNymError.invalidUrl
        }

        let locations = try getGatewayCountries(
            apiUrl: apiURL,
            explorerUrl: explorerURL,
            exitOnly: true
        )
        let newExitCountries = convertToCountriesAndSort(from: locations)

        entryCountries = nil
        exitCountries = newExitCountries
        isLoading = false
    }

    func fetchLowLatencyEntryCountry() {
        guard
            let apiURL = URL(string: Constants.apiUrl.rawValue),
            let explorerURL = URL(string: Constants.explorerUrl.rawValue),
            let location = try? getLowLatencyEntryCountry(apiUrl: apiURL, explorerUrl: explorerURL)
        else {
            return
        }

        lowLatencyCountry = Country(name: location.countryName, code: location.twoLetterIsoCountryCode)
    }
}

private extension CountriesManager {
    func convertToCountriesAndSort(from locations: [Location]) -> [Country] {
        locations.map {
            Country(name: $0.countryName, code: $0.twoLetterIsoCountryCode)
        }
        .sorted { $0.name < $1.name }
    }
}
