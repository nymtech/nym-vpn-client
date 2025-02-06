import Combine
import SwiftUI
import AppSettings
import AppVersionProvider
import ConfigurationManager
import ConnectionManager
import CredentialsManager
import ExternalLinkManager
import UIComponents

public class SettingsViewModel: SettingsFlowState {
    private let appSettings: AppSettings
    private let configurationManager: ConfigurationManager
    private let connectionManager: ConnectionManager
    private let credentialsManager: CredentialsManager
    private let externalLinkManager: ExternalLinkManager

    private var cancellables = Set<AnyCancellable>()
    private var deviceIdentifier: String? {
        guard let deviceIdentifier = credentialsManager.deviceIdentifier else { return nil }
        return "settings.deviceId".localizedString + deviceIdentifier
    }

    let settingsTitle = "settings".localizedString

    @Published var isLogoutConfirmationDisplayed = false
    @Published var sections: [SettingsSection] = []

    var isValidCredentialImported: Bool {
        credentialsManager.isValidCredentialImported
    }

    var logoutDialogConfiguration: ActionDialogConfiguration {
        ActionDialogConfiguration(
            iconImageName: "exclamationmark.circle",
            titleLocalizedString: "settings.logoutTitle".localizedString,
            subtitleLocalizedString: "settings.logoutSubtitle".localizedString,
            yesLocalizedString: "cancel".localizedString,
            noLocalizedString: "settings.logout".localizedString,
            noAction: { [weak self] in
                Task {
                    await self?.logout()
                }
            }
        )
    }

    public init(
        path: Binding<NavigationPath>,
        appSettings: AppSettings = .shared,
        configurationManager: ConfigurationManager = .shared,
        connectionManager: ConnectionManager = .shared,
        credentialsManager: CredentialsManager = .shared,
        externalLinkManager: ExternalLinkManager = .shared
    ) {
        self.appSettings = appSettings
        self.configurationManager = configurationManager
        self.connectionManager = connectionManager
        self.credentialsManager = credentialsManager
        self.externalLinkManager = externalLinkManager
        super.init(path: path)
        setup()
    }

    func navigateHome() {
        path = .init()
    }

    func appVersion() -> String {
        AppVersionProvider.appVersion()
    }

    func navigateToAddCredentialsOrCredential() {
        guard configurationManager.isSantaClaus else { return }
        path.append(SettingLink.addCredentials)
    }

    func navigateToSantasMenu() {
        path.append(SettingLink.santasMenu)
    }
}

private extension SettingsViewModel {
    func navigateToTheme() {
        path.append(SettingLink.theme)
    }

    func navigateToLogs() {
        path.append(SettingLink.logs)
    }

    func navigateToSupportAndFeedback() {
        path.append(SettingLink.support)
    }

    func navigateToLegal() {
        path.append(SettingLink.legal)
    }

    func navigateToAccount() {
        try? externalLinkManager.openExternalURL(urlString: configurationManager.accountLinks?.account)
    }
}

// MARK: - Setup -
private extension SettingsViewModel {
    func setup() {
        setupAppSettingsObservers()
        configureSections()
    }

    func setupAppSettingsObservers() {
        appSettings.$isCredentialImportedPublisher.sink { [weak self] _ in
            self?.configureSections()
        }
        .store(in: &cancellables)
    }

    func configureSections() {
        var newSections = [SettingsSection]()
        if appSettings.isCredentialImported {
            newSections.append(accountSection())
        }
        newSections.append(
            contentsOf: [
                feedbackSection(),
                themeSection(),
                legalSection()
            ]
        )
        if appSettings.isCredentialImported {
            newSections.append(logoutSection())
        }
        sections = newSections
    }
}

// MARK: - Actions -
private extension SettingsViewModel {
    func logout() async {
        await connectionManager.disconnectBeforeLogout()
        try? await credentialsManager.removeCredential()
    }
}

// MARK: - Sections -
private extension SettingsViewModel {
    func accountSection() -> SettingsSection {
        .account(
            viewModels: [
                SettingsListItemViewModel(
                    accessory: .externalLink,
                    title: "settings.account".localizedString,
                    subtitle: deviceIdentifier,
                    imageName: "person",
                    action: { [weak self] in
                        self?.navigateToAccount()
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
                    title: "settings.supportAndFeedback".localizedString,
                    imageName: "support",
                    action: { [weak self] in
                        self?.navigateToSupportAndFeedback()
                    }
                ),
                SettingsListItemViewModel(
                    accessory: .arrow,
                    title: "logs".localizedString,
                    imageName: "logs",
                    action: { [weak self] in
                        self?.navigateToLogs()
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

    func logoutSection() -> SettingsSection {
        .logout(
            viewModels: [
                SettingsListItemViewModel(
                    accessory: .empty,
                    title: "settings.logout".localizedString,
                    action: { [weak self] in
                        self?.isLogoutConfirmationDisplayed = true
                    }
                )
            ]
        )
    }
}
