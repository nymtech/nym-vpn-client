import AppSettings
import ConfigurationManager
import CountriesManager
import CountriesManagerTypes

public final class ConnectionStorage {
    public static let shared = ConnectionStorage()

    private let appSettings: AppSettings
    private let configurationManager: ConfigurationManager
    private let countriesManager: CountriesManager

    private var countryType: CountryType {
        connectionType() == .wireguard ? .vpn : .entry
    }

    public init(
        appSettings: AppSettings = .shared,
        configurationManager: ConfigurationManager = .shared,
        countriesManager: CountriesManager = .shared
    ) {
        self.appSettings = appSettings
        self.configurationManager = configurationManager
        self.countriesManager = countriesManager
    }

    func connectionType() -> ConnectionType {
        if let typeValue = appSettings.connectionType,
           let connectionType = ConnectionType(rawValue: typeValue) {
            return connectionType
        } else {
            return ConnectionType.wireguard
        }
    }

    /// Manipulates gateway if last parameter does not exist anymore.
    /// Example: Checks if country exists, if not returns Switzerland, if Switzerland does not exist - first country.
    /// - Returns: EntryGateway
    func entryGateway() -> EntryGateway {
        let jsonString = appSettings.entryGateway ?? ""
        guard let gateway = EntryGateway.from(jsonString: jsonString)
        else {
            // Fallback to Switzerland or first country
            return .country(fallbackCountry(countryType: countryType))
        }

        switch gateway {
        case let .country(country):
            let existingCountry = existingCountry(with: country.code, countryType: countryType)
            return .country(existingCountry)
        case let .lowLatencyCountry(country):
            let country = existingCountry(with: country.code, countryType: countryType)
            return .lowLatencyCountry(country)
        case let .gateway(identifier):
            return .gateway(identifier)
        case .randomLowLatency:
            return .randomLowLatency
        case .random:
            return .random
        }
    }

    /// Manipulates router if last parameter does not exist anymore.
    /// Example: Checks if country exists, if not returns Switzerland, if Switzerland does not exist - first country.
    /// - Returns: ExitRouter
    func exitRouter() -> ExitRouter {
        let jsonString = appSettings.exitRouter ?? ""
        guard let router = ExitRouter.from(jsonString: jsonString)
        else {
            return .country(fallbackCountry(countryType: countryType))
        }

        switch router {
        case let .country(country):
            let existingCountry = existingCountry(with: country.code, countryType: countryType)
            return .country(existingCountry)
        case let .gateway(identifier):
            return .gateway(identifier)
        }
    }
}

private extension ConnectionStorage {
    /// Checks if selected gateway country exists. If not - returns first country from the country list, if no countries present - returns Switzerland
    /// - Parameter countryCode: String
    /// - Parameter isEntryHop: Bool. Determines from which country array(entry/exit) to return the country from
    /// - Returns: String with countryCode
    func existingCountry(with countryCode: String, countryType: CountryType) -> Country {
        let country = countriesManager.country(with: countryCode, countryType: countryType)

        if let country {
            return country
        } else {
            return fallbackCountry(countryType: countryType)
        }
    }

    func fallbackCountry(countryType: CountryType) -> Country {
        let fallbackCountry = Country(name: "Switzerland", code: "CH")
        switch countryType {
        case .entry:
            if countriesManager.entryCountries.contains(where: { $0.code == "CH" }) {
                return fallbackCountry
            } else if let country = countriesManager.entryCountries.first {
                return country
            }
        case .exit:
            if countriesManager.exitCountries.contains(where: { $0.code == "CH" }) {
                return fallbackCountry
            } else if let country = countriesManager.exitCountries.first {
                return country
            }
        case .vpn:
            if countriesManager.vpnCountries.contains(where: { $0.code == "CH" }) {
                return fallbackCountry
            } else if let country = countriesManager.vpnCountries.first {
                return country
            }
        }
        return fallbackCountry
    }
}
