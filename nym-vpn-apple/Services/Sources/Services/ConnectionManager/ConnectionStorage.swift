import AppSettings
import CountriesManager

public final class ConnectionStorage {
    public static let shared = ConnectionStorage()

    private let appSettings: AppSettings
    private let countriesManager: CountriesManager

    public init(
        appSettings: AppSettings = AppSettings.shared,
        countriesManager: CountriesManager = CountriesManager.shared
    ) {
        self.appSettings = appSettings
        self.countriesManager = countriesManager
    }

    func connectionType() -> ConnectionType {
        if let typeValue = appSettings.connectionType,
           let connectionType = ConnectionType(rawValue: typeValue) {
            return connectionType
        } else {
            return ConnectionType.mixnet5hop
        }
    }

    func entryGateway() -> EntryGateway {
        if !appSettings.isEntryLocationSelectionOn {
            return .random
        }

        let countryType: CountryType = connectionType() == .wireguard ? .vpn : .entry
        let country = countriesManager.country(with: appSettings.entryCountryCode, countryType: countryType)

        if !appSettings.entryCountryCode.isEmpty && country != nil {
            return .country(code: existingCountryCode(with: appSettings.entryCountryCode, countryType: .entry))
        } else {
            guard let entryCountry = self.countriesManager.entryCountries.first
            else {
                return .country(code: "CH")
            }
            return .country(code: entryCountry.code)
        }
    }

    func exitRouter() -> ExitRouter {
        let countryType: CountryType = connectionType() == .wireguard ? .vpn : .exit
        let country = countriesManager.country(with: appSettings.entryCountryCode, countryType: countryType)

        if !appSettings.exitCountryCode.isEmpty && country != nil {
            return .country(code: existingCountryCode(with: appSettings.exitCountryCode, countryType: .exit))
        } else {
            guard let exitCountry = self.countriesManager.exitCountries.first
            else {
                return .country(code: "CH")
            }
            return .country(code: exitCountry.code)
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
            if let country = countriesManager.entryCountries.first {
                return country.code
            } else {
                return "CH"
            }
        case .exit:
            if let country = countriesManager.exitCountries.first {
                return country.code
            } else {
                return "CH"
            }
        case .vpn:
            if let country = countriesManager.vpnCountries.first {
                return country.code
            } else {
                return "CH"
            }
        }
    }
}
