import Foundation
import AppVersionProvider
import ConfigurationManager
import Theme

public struct SettingsListViewModel {
    private let appVersion: String
    private let configurationManager: ConfigurationManager

    let sections: [SettingsSection]

    var versionTitle: String {
        "\("version".localizedString) \(appVersion) (\(AppVersionProvider.libVersion))"
    }

    var isTestFlight: Bool {
        configurationManager.isTestFlight
    }

    var envs: [Env] {
        Env.allCases
    }

    public init(
        sections: [SettingsSection],
        appVersion: String,
        configurationManager: ConfigurationManager
    ) {
        self.sections = sections
        self.appVersion = appVersion
        self.configurationManager = configurationManager
    }

    func changeEnvironment(to env: Env) {
        configurationManager.updateEnv(to: env)
    }
}
