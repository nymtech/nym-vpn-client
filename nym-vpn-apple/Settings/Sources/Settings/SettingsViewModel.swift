import SwiftUI
import AppSettings
import AppVersionProvider
import CredentialsManager
import UIComponents

public class SettingsViewModel: SettingsFlowState {
    private let appSettings: AppSettings
    private let credentialsManager: CredentialsManager

    let settingsTitle = "settings".localizedString

    var sections: [SettingsSection] {
        [
            connectionSection(),
            themeSection(),
            feedbackSection(),
            legalSection()
        ]
    }

    var isValidCredentialImported: Bool {
        credentialsManager.isValidCredentialImported
    }

    public init(
        path: Binding<NavigationPath>,
        appSettings: AppSettings = AppSettings.shared,
        credentialsManager: CredentialsManager = CredentialsManager.shared
    ) {
        self.appSettings = appSettings
        self.credentialsManager = credentialsManager
        super.init(path: path)
    }

    func navigateHome() {
        path = .init()
    }

    func appVersion() -> String {
        AppVersionProvider.appVersion()
    }

    func navigateToAddCredentialsOrCredential() {
        path.append(SettingsLink.addCredentials)
    }
}

private extension SettingsViewModel {
    func navigateToTheme() {
        path.append(SettingsLink.theme)
    }

    func navigateToLogs() {
        path.append(SettingsLink.logs)
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
                    accessory: .arrow,
                    title: "logs".localizedString,
                    imageName: "logs",
                    action: { [weak self] in
                        self?.navigateToLogs()
                    }
                )
            ]
        )
    }

    func themeSection() -> SettingsSection {
        .theme(
            viewModels: [
                SettingsListItemViewModel(
                    accessory: .arrow,
                    title: "settings.appearance".localizedString,
                    imageName: "appearance",
                    action: { [weak self] in
                        self?.navigateToTheme()
                    }
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
