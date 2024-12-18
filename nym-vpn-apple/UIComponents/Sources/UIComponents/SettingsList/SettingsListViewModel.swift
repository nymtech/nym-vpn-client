import SwiftUI
import AppVersionProvider
import ConfigurationManager
import Device
import Theme

public struct SettingsListViewModel {
    private let appVersion: String
    private let configurationManager: ConfigurationManager
    private let navigateToSantasMenuAction: (() -> Void)

    let sections: [SettingsSection]

    var versionTitle: String {
        "\("version".localizedString) \(appVersion) (\(AppVersionProvider.libVersion))"
    }

    public init(
        sections: [SettingsSection],
        appVersion: String,
        configurationManager: ConfigurationManager,
        navigateToSantasMenuAction: @escaping (() -> Void)
    ) {
        self.sections = sections
        self.appVersion = appVersion
        self.configurationManager = configurationManager
        self.navigateToSantasMenuAction = navigateToSantasMenuAction
    }

    public func navigateToSantasMenu() {
        guard configurationManager.isSantaClaus else { return }
        navigateToSantasMenuAction()
    }
}
