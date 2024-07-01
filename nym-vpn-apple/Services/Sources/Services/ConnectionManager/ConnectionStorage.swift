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
            return .randomLowLatency
        }

        if !appSettings.entryCountryCode.isEmpty {
            return .country(code: appSettings.entryCountryCode)
        } else {
            guard let entryCountry = self.countriesManager.entryCountries.first
            else {
                return .country(code: "CH")
            }
            return .country(code: entryCountry.code)
        }
    }

    func exitRouter() -> ExitRouter {
        if !appSettings.exitCountryCode.isEmpty {
            return .country(code: appSettings.exitCountryCode)
        } else {
            guard let exitCountry = self.countriesManager.exitCountries.first
            else {
                return .country(code: "CH")
            }
            return .country(code: exitCountry.code)
        }
    }
}
