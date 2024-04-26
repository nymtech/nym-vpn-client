import Foundation

final class LastHopStore {
    var countries: [Country]?
    var lowLatencyCountry: Country?
    var lastFetchDate: Date

    init(countries: [Country]? = nil, lowLatencyCountry: Country? = nil, lastFetchDate: Date) {
        self.countries = countries
        self.lowLatencyCountry = lowLatencyCountry
        self.lastFetchDate = lastFetchDate
    }
}
