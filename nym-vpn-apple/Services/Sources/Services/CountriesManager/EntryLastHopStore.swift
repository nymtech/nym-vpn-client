import Foundation

final class EntryLastHopStore {
    var entryCountries: [Country]?
    var exitCountries: [Country]?
    var lowLatencyCountry: Country?
    var lastFetchDate: Date

    init(
        entryCountries: [Country]? = nil,
        exitCountries: [Country]? = nil,
        lowLatencyCountry: Country? = nil,
        lastFetchDate: Date
    ) {
        self.entryCountries = entryCountries
        self.exitCountries = exitCountries
        self.lowLatencyCountry = lowLatencyCountry
        self.lastFetchDate = lastFetchDate
    }
}
