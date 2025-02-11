import Foundation
#if os(macOS)
import ServiceManagement
#endif
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
        #if os(macOS)
        migrateDaemon()
        #endif
    }
}

private extension Migrations {
    func migrateToMainnet() {
        guard appSettings.currentEnv != "mainnet",
              !configurationManager.isSantaClaus
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

#if os(macOS)
    func migrateDaemon() {
        guard let url = URL(string: "/Library/LaunchDaemons/net.nymtech.vpn.helper.plist"),
              SMAppService.statusForLegacyPlist(at: url) == .enabled
        else {
            return
        }

        let domain = kSMDomainSystemLaunchd
        var authRef: AuthorizationRef?
        let status = AuthorizationCreate(nil, nil, [], &authRef)

        guard status == errAuthorizationSuccess,
              let authorization = authRef
        else {
            return
        }

        var cfError: Unmanaged<CFError>?
        SMJobRemove(domain, "net.nymtech.vpn.helper" as CFString, authorization, true, &cfError)
    }
#endif
}
