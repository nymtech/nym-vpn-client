import Foundation

public final class CountryStore: Codable {
    public typealias RawValue = String

    public var entryCountries: [NymCountry]
    public var exitCountries: [NymCountry]
    public var vpnCountries: [NymCountry]
    public var lastFetchDate: Date?

    public init(
        lastFetchDate: Date? = nil,
        entryCountries: [NymCountry] = [],
        exitCountries: [NymCountry] = [],
        vpnCountries: [NymCountry] = []
    ) {
        self.lastFetchDate = lastFetchDate
        self.entryCountries = entryCountries
        self.exitCountries = exitCountries
        self.vpnCountries = vpnCountries
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)

        try container.encode(entryCountries, forKey: .entryCountries)
        try container.encode(exitCountries, forKey: .exitCountries)
        try container.encode(vpnCountries, forKey: .vpnCountries)

        if let lastFetchDate = lastFetchDate {
            try container.encode(lastFetchDate.timeIntervalSince1970, forKey: .lastFetchDate)
        }
    }

    public required init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)

        entryCountries = try container.decode([NymCountry].self, forKey: .entryCountries)
        exitCountries = try container.decode([NymCountry].self, forKey: .exitCountries)
        vpnCountries = try container.decode([NymCountry].self, forKey: .vpnCountries)

        if let timeInterval = try? container.decode(Double.self, forKey: .lastFetchDate) {
            lastFetchDate = Date(timeIntervalSince1970: timeInterval)
        } else {
            lastFetchDate = nil
        }
    }

    public var rawValue: RawValue {
        guard let data = try? JSONEncoder().encode(self),
              let result = String(data: data, encoding: .utf8)
        else {
            return ""
        }
        return result
    }

    public convenience init?(rawValue: RawValue) {
        guard let data = rawValue.data(using: .utf8),
              let countryStore = try? JSONDecoder().decode(CountryStore.self, from: data)
        else {
            return nil
        }
        self.init(
            lastFetchDate: countryStore.lastFetchDate,
            entryCountries: countryStore.entryCountries,
            exitCountries: countryStore.exitCountries,
            vpnCountries: countryStore.vpnCountries
        )
    }

    private enum CodingKeys: String, CodingKey {
        case entryCountries
        case exitCountries
        case vpnCountries
        case lastFetchDate
    }
}
