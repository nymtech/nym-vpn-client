import Foundation
#if os(macOS)
import ServiceManagement
#endif
import AppSettings
import ConfigurationManager
import ConnectionTypes
import ConnectionTypes

@MainActor public final class Migrations {
    private let appSettings: AppSettings
    private let configurationManager: ConfigurationManager

    public static let shared = Migrations(
        appSettings: .shared,
        configurationManager: .shared
    )

    private init(
        appSettings: AppSettings,
        configurationManager: ConfigurationManager
    ) {
        self.appSettings = appSettings
        self.configurationManager = configurationManager
    }

    public func setup() {
        migrateToMainnet()
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
}
