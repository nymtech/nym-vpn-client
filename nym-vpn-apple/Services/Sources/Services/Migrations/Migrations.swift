import AppSettings
import ConfigurationManager
import ConnectionTypes
import CountriesManagerTypes
import CountriesManager

public final class Migrations {
    private let appSettings: AppSettings
    private let configurationManager: ConfigurationManager
    private let countriesManager: CountriesManager

    public static let shared = Migrations(
        appSettings: .shared,
        configurationManager: .shared,
        countriesManager: .shared
    )

    private init(
        appSettings: AppSettings,
        configurationManager: ConfigurationManager,
        countriesManager: CountriesManager
    ) {
        self.appSettings = appSettings
        self.configurationManager = configurationManager
        self.countriesManager = countriesManager
    }

    public func setup() {
        migrateToMainnet()
        migrateCountryNames()
    }
}

private extension Migrations {
    func migrateToMainnet() {
        guard appSettings.currentEnv != "mainnet",
              !configurationManager.isTestFlight
        else {
            return
        }
        Task { @MainActor in
            appSettings.currentEnv = "mainnet"
        }
    }

    func migrateCountryNames() {
        // Introduced in v1.6.0
        if let entryCountry = countriesManager.country(with: appSettings.entryCountryCode) {
            appSettings.entryGateway = EntryGateway.country(entryCountry).toJson()
            appSettings.entryCountryCode = ""
        }
        if let exitCountry = countriesManager.country(with: appSettings.exitCountryCode) {
            appSettings.entryGateway = EntryGateway.country(exitCountry).toJson()
            appSettings.exitCountryCode = ""
        }
    }
}
