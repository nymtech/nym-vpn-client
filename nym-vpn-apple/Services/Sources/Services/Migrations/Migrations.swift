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
    }

    public var isMacOSWgDisabled: Bool {
        appSettings.isMacOS && configurationManager.appVersion == "1.1.1"
    }

    public func setup() {
        macOSWgDisabledMigration()
    }
}

private extension Migrations {
    func macOSWgDisabledMigration() {
        guard isMacOSWgDisabled else { return }
        appSettings.connectionType = nil
    }
}
