import Foundation

final class CountryStore {
    var entryCountries: [Country]
    var exitCountries: [Country]
    var vpnCountries: [Country]
    var lowLatencyCountry: Country?
    var lastFetchDate: Date?

    init(
        lastFetchDate: Date? = nil,
        entryCountries: [Country] = [],
        exitCountries: [Country] = [],
        vpnCountries: [Country] = [],
        lowLatencyCountry: Country? = nil
    ) {
        self.lastFetchDate = lastFetchDate
        self.entryCountries = entryCountries
        self.exitCountries = exitCountries
        self.vpnCountries = vpnCountries
        self.lowLatencyCountry = lowLatencyCountry
    }
}
