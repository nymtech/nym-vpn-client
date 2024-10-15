import AppSettings
import ConfigurationManager

public final class Migrations {
    private let appSettings: AppSettings
    private let configurationManager: ConfigurationManager

    public static let shared = Migrations(
        appSettings: AppSettings.shared,
        configurationManager: ConfigurationManager.shared
    )

    private init(
        appSettings: AppSettings,
        configurationManager: ConfigurationManager
    ) {
        self.appSettings = appSettings
        self.configurationManager = configurationManager

        setup()
    }

    public func setup() {
        migrateToMainnet()
    }
}

private extension Migrations {
    func migrateToMainnet() {
        guard appSettings.currentEnv != "mainnet" else { return }
        appSettings.currentEnv = "mainnet"
    }
}
