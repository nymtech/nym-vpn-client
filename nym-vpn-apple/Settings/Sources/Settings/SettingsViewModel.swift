import Combine
import SwiftUI
import AppSettings
import AppVersionProvider
import CredentialsManager
import UIComponents

public class SettingsViewModel: SettingsFlowState {
    private let appSettings: AppSettings
    private let credentialsManager: CredentialsManager

    private var cancellables = Set<AnyCancellable>()

    let settingsTitle = "settings".localizedString

    @Published var isLogoutConfirmationDisplayed = false

    @Published var sections: [SettingsSection] = []

    var isValidCredentialImported: Bool {
        credentialsManager.isValidCredentialImported
    }

    var logoutDialogConfiguration: ActionDialogConfiguration {
        ActionDialogConfiguration(
            titleLocalizedString: "settings.logout".localizedString,
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
        appSettings: AppSettings = AppSettings.shared,
        credentialsManager: CredentialsManager = CredentialsManager.shared
    ) {
        self.appSettings = appSettings
        self.credentialsManager = credentialsManager
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
        var newSections = [
            connectionSection(),
            themeSection(),
            feedbackSection(),
            legalSection()
        ]
        if appSettings.isCredentialImported {
            newSections.append(logoutSection())
        }
        sections = newSections
    }
}

// MARK: - Actions -
private extension SettingsViewModel {
    func logout() async {
        try? await credentialsManager.removeCredential()
        // TODO: check if can login/logout
    }
}

// MARK: - Sections -
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
