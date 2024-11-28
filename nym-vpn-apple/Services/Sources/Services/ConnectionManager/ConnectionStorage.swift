import AppSettings
import ConfigurationManager
import CountriesManager

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

    func entryGateway() -> EntryGateway {
        if !appSettings.entryCountryCode.isEmpty {
            return .country(code: existingCountryCode(with: appSettings.entryCountryCode, countryType: countryType))
        } else {
            return .country(code: fallbackCountryCode(countryType: countryType))
        }
    }

    func exitRouter() -> ExitRouter {
        if !appSettings.exitCountryCode.isEmpty {
            return .country(code: existingCountryCode(with: appSettings.exitCountryCode, countryType: countryType))
        } else {
            return .country(code: fallbackCountryCode(countryType: countryType))
        }
    }
}

private extension ConnectionStorage {
    /// Checks if selected gateway country exists. If not - returns first country from the country list, if no countries present - returns Switzerland
    /// - Parameter countryCode: String
    /// - Parameter isEntryHop: Bool. Determines from which country array(entry/exit) to return the country from
    /// - Returns: String with countryCode
    func existingCountryCode(with countryCode: String, countryType: CountryType) -> String {
        let country = countriesManager.country(with: countryCode, countryType: countryType)

        if let country {
            return country.code
        } else {
            return fallbackCountryCode(countryType: countryType)
        }
    }

    func fallbackCountryCode(countryType: CountryType) -> String {
        switch countryType {
        case .entry:
            if countriesManager.entryCountries.contains(where: { $0.code == "CH" }) {
                return "CH"
            } else if let country = countriesManager.entryCountries.first {
                return country.code
            }
        case .exit:
            if countriesManager.exitCountries.contains(where: { $0.code == "CH" }) {
                return "CH"
            } else if let country = countriesManager.exitCountries.first {
                return country.code
            }
        case .vpn:
            if countriesManager.vpnCountries.contains(where: { $0.code == "CH" }) {
                return "CH"
            } else if let country = countriesManager.vpnCountries.first {
                return country.code
            }
        }
        return "CH"
    }
}
