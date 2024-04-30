import SwiftUI
import AppSettings
import AppVersionProvider
import UIComponents

public class SettingsViewModel: SettingsFlowState {
    private var appSettings: AppSettings

    let settingsTitle = "settings".localizedString

    var shouldShowAddCredentials: Bool {
        !appSettings.isCredentialImported
    }

    var sections: [SettingsSection] {
        [
            connectionSection(),
            themeSection(),
//            logsSection(),
            feedbackSection(),
            legalSection()
        ]
    }

    public init(path: Binding<NavigationPath>, appSettings: AppSettings = AppSettings.shared) {
        self.appSettings = appSettings
        super.init(path: path)
    }

    func navigateHome() {
        path = .init()
    }

    func appVersion() -> String {
        AppVersionProvider.appVersion()
    }

    func navigateToAddCredentials() {
        path.append(SettingsLink.addCredentials)
    }
}

private extension SettingsViewModel {
    func navigateToTheme() {
        path.append(SettingsLink.theme)
    }

    func navigateToFeedback() {
        path.append(SettingsLink.feedback)
    }

    func navigateToSupport() {
        path.append(SettingsLink.support)
    }

    func navigateToLegal() {
        path.append(SettingsLink.legal)
    }
}

private extension SettingsViewModel {
    func connectionSection() -> SettingsSection {
        .connection(
            viewModels: [
//                SettingsListItemViewModel(
//                    accessory: .arrow,
//                    title: "autoConnectTitle".localizedString,
//                    subtitle: "autoConnectSubtitle".localizedString,
//                    imageName: "autoConnect",
//                    action: {}
//                ),
                SettingsListItemViewModel(
                    accessory: .toggle(
                        viewModel: ToggleViewModel(isOn: appSettings.isEntryLocationSelectionOn) { [weak self] isOn in
                            self?.appSettings.isEntryLocationSelectionOn = isOn
                        }
                    ),
                    title: "entryLocationTitle".localizedString,
                    subtitle: "entryLocationSubtitle".localizedString,
                    imageName: "entryHop",
                    action: {}
                )
            ]
        )
    }

    func themeSection() -> SettingsSection {
        .theme(
            viewModels: [
                SettingsListItemViewModel(
                    accessory: .arrow,
                    title: "displayTheme".localizedString,
                    imageName: "displayTheme",
                    action: { [weak self] in
                        self?.navigateToTheme()
                    }
                )
            ]
        )
    }

    func logsSection() -> SettingsSection {
        .logs(
            viewModels: [
                SettingsListItemViewModel(
                    accessory: .arrow,
                    title: "logs".localizedString,
                    imageName: "logs",
                    action: {}
                )
            ]
        )
    }

    func feedbackSection() -> SettingsSection {
        .feedback(
            viewModels: [
                SettingsListItemViewModel(
                    accessory: .arrow,
                    title: "feedback".localizedString,
                    imageName: "feedback",
                    action: { [weak self] in
                        self?.navigateToFeedback()
                    }
                ),
                SettingsListItemViewModel(
                    accessory: .arrow,
                    title: "support".localizedString,
                    imageName: "support",
                    action: { [weak self] in
                        self?.navigateToSupport()
                    }
                ),
                SettingsListItemViewModel(
                    accessory: .toggle(
                        viewModel: ToggleViewModel(
                            isOn: appSettings.isErrorReportingOn,
                            action: { [weak self] isOn in
                                self?.appSettings.isErrorReportingOn = isOn
                            }
                        )
                    ),
                    title: "settings.anonymousErrorReports.title".localizedString,
                    subtitle: "settings.anonymousErrorReports.subtitle".localizedString,
                    imageName: "errorReport",
                    action: {}
                )
            ]
        )
    }

    func legalSection() -> SettingsSection {
        .legal(
            viewModels: [
                SettingsListItemViewModel(
                    accessory: .arrow,
                    title: "legal".localizedString,
                    action: { [weak self] in
                        self?.navigateToLegal()
                    }
                )
            ]
        )
    }
}
