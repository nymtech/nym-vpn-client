import Foundation

final class EntryLastHopStore {
    var entryCountries: [Country]
    var exitCountries: [Country]
    var lowLatencyCountry: Country?
    var lastFetchDate: Date?

    init(
        lastFetchDate: Date? = nil,
        entryCountries: [Country] = [],
        exitCountries: [Country] = [],
        lowLatencyCountry: Country? = nil
    ) {
        self.lastFetchDate = lastFetchDate
        self.entryCountries = entryCountries
        self.exitCountries = exitCountries
        self.lowLatencyCountry = lowLatencyCountry
    }
}
