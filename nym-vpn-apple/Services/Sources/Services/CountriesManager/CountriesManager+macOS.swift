#if os(macOS)

// MARK: - Fetching -
extension CountriesManager {
    func fetchEntryExitCountries() throws {
//        guard
//            let apiURL = URL(string: Constants.apiUrl.rawValue),
//            let explorerURL = URL(string: Constants.explorerURL.rawValue),
//            let harbourURL = URL(string: Constants.harbourURL.rawValue)
//        else {
//            throw GeneralNymError.invalidUrl
//        }
//
//        let locations = try getGatewayCountries(
//            apiUrl: apiURL,
//            explorerUrl: explorerURL,
//            harbourMasterUrl: harbourURL,
//            exitOnly: false
//        )
//        let newEntryCountries = convertToCountriesAndSort(from: locations)
//        let newExitCountries = convertToCountriesAndSort(from: locations)
//
//        entryLastHopStore.entryCountries = newEntryCountries
//        entryLastHopStore.exitCountries = newExitCountries
//        entryLastHopStore.lastFetchDate = Date()
//
//        entryCountries = newEntryCountries
//        exitCountries = newExitCountries
//        isLoading = false
    }

    func fetchExitCountries() throws {
//        guard
//            let apiURL = URL(string: Constants.apiUrl.rawValue),
//            let explorerURL = URL(string: Constants.explorerURL.rawValue),
//            let harbourURL = URL(string: Constants.harbourURL.rawValue)
//        else {
//            throw GeneralNymError.invalidUrl
//        }
//
//        let locations = try getGatewayCountries(
//            apiUrl: apiURL,
//            explorerUrl: explorerURL,
//            harbourMasterUrl: harbourURL,
//            exitOnly: true
//        )
//        let newExitCountries = convertToCountriesAndSort(from: locations)
//
//        lastHopStore.countries = newExitCountries
//        lastHopStore.lastFetchDate = Date()
//
//        entryCountries = nil
//        exitCountries = newExitCountries
//        isLoading = false
//        updateHasCountries()
    }

    func fetchLowLatencyEntryCountry() {
//        guard
//            let apiURL = URL(string: Constants.apiUrl.rawValue),
//            let explorerURL = URL(string: Constants.explorerURL.rawValue),
//            let harbourURL = URL(string: Constants.harbourURL.rawValue),
//            let location = try? getLowLatencyEntryCountry(
//                apiUrl: apiURL,
//                explorerUrl: explorerURL,
//                harbourMasterUrl: harbourURL
//            )
//        else {
//            return
//        }
//        entryLastHopStore.lowLatencyCountry = lowLatencyCountry
//        lastHopStore.lowLatencyCountry = lowLatencyCountry
//        lowLatencyCountry = Country(name: location.countryName, code: location.twoLetterIsoCountryCode)
//        updateHasCountries()
    }
}
#endif
