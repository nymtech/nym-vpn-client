import Combine
import SwiftUI
import AppSettings
import AppVersionProvider
import ConfigurationManager
import ConnectionManager
import CredentialsManager
import ExternalLinkManager
#if os(macOS)
import HelperInstall
import HelperManager
#endif
import UIComponents

public class SettingsViewModel: SettingsFlowState {
    private let appSettings: AppSettings
    private let configurationManager: ConfigurationManager
    private let connectionManager: ConnectionManager
    private let externalLinkManager: ExternalLinkManager
#if os(macOS)
    private let helperManager: HelperManager
#endif

    private var cancellables = Set<AnyCancellable>()
    private var deviceIdentifier: String? {
        guard let deviceIdentifier = credentialsManager.deviceIdentifier else { return nil }
        return "settings.deviceId".localizedString + deviceIdentifier
    }

    let credentialsManager: CredentialsManager
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

#if os(iOS)
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
#elseif os(macOS)
    public init(
        path: Binding<NavigationPath>,
        appSettings: AppSettings = .shared,
        configurationManager: ConfigurationManager = .shared,
        connectionManager: ConnectionManager = .shared,
        credentialsManager: CredentialsManager = .shared,
        externalLinkManager: ExternalLinkManager = .shared,
        helperManager: HelperManager = .shared
    ) {
        self.appSettings = appSettings
        self.configurationManager = configurationManager
        self.connectionManager = connectionManager
        self.credentialsManager = credentialsManager
        self.externalLinkManager = externalLinkManager
        self.helperManager = helperManager
        super.init(path: path)
        setup()
    }
#endif

    func appVersion() -> String {
        AppVersionProvider.appVersion()
    }

    @MainActor func navigateHome() {
        path = .init()
    }

    @MainActor func navigateToAddCredentialsOrCredential() {
#if os(macOS)
        guard !helperManager.isInstallNeeded()
        else {
            navigateToInstallHelper()
            return
        }
#endif
        if credentialsManager.isValidCredentialImported {
            navigateToAccount()
        } else {
            path.append(SettingLink.addCredentials)
        }
    }

    @MainActor func navigateToSantasMenu() {
        guard configurationManager.isSantaClaus else { return }
        path.append(SettingLink.santasMenu)
    }
}

private extension SettingsViewModel {
#if os(macOS)
    @MainActor func navigateToAppMode() {
        path.append(SettingLink.appMode)
    }
#endif

    @MainActor func navigateToTheme() {
        path.append(SettingLink.theme)
    }

    @MainActor func navigateToLogs() {
        path.append(SettingLink.logs)
    }

    @MainActor func navigateToSupportAndFeedback() {
        path.append(SettingLink.support)
    }

    @MainActor func navigateToLegal() {
        path.append(SettingLink.legal)
    }

    @MainActor func navigateToAccount() {
        try? externalLinkManager.openExternalURL(urlString: configurationManager.accountLinks?.account)
    }

#if os(macOS)
    @MainActor func navigateToInstallHelper() {
        let action = HelperAfterInstallAction { [weak self] in
            self?.navigateToAddCredentialsOrCredential()
        }
        path.append(SettingLink.installHelper(afterInstallAction: action))
    }
#endif
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
                killswitchSection(),
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
                        Task { @MainActor in
                            self?.navigateToAccount()
                        }
                    }
                )
            ]
        )
    }

    func themeSection() -> SettingsSection {
        var viewModels = [
            SettingsListItemViewModel(
                accessory: .arrow,
                title: "settings.appearance".localizedString,
                imageName: "appearance",
                action: { [weak self] in
                    Task { @MainActor in
                        self?.navigateToTheme()
                    }
                }
            )
        ]
#if os(macOS)
        viewModels.append(
            SettingsListItemViewModel(
                accessory: .arrow,
                title: "settings.appMode".localizedString,
                systemImageName: "menubar.dock.rectangle",
                action: { [weak self] in
                    Task { @MainActor in
                        self?.navigateToAppMode()
                    }
                }
            )
        )
#endif
        return .theme(viewModels: viewModels)
    }

    func feedbackSection() -> SettingsSection {
        .feedback(
            viewModels: [
                SettingsListItemViewModel(
                    accessory: .arrow,
                    title: "settings.supportAndFeedback".localizedString,
                    imageName: "support",
                    action: { [weak self] in
                        Task { @MainActor in
                            self?.navigateToSupportAndFeedback()
                        }
                    }
                ),
                SettingsListItemViewModel(
                    accessory: .arrow,
                    title: "logs".localizedString,
                    imageName: "logs",
                    action: { [weak self] in
                        Task { @MainActor in
                            self?.navigateToLogs()
                        }
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

    func killswitchSection() -> SettingsSection {
        .killSwitch(
            viewModels: [
                SettingsListItemViewModel(
                    accessory: .toggle(viewModel: ToggleViewModel(isOn: true, isDisabled: true)),
                    title: "settings.killswitch.title".localizedString,
                    subtitle: "settings.killswitch.subtitle".localizedString,
                    systemImageName: "power",
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
                        Task { @MainActor in
                            self?.navigateToLegal()
                        }
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

extension SettingsViewModel {
    func copyToPasteboard(text: String) {
#if os(iOS)
        UIPasteboard.general.string = text
#elseif os(macOS)
        NSPasteboard.general.prepareForNewContents()
        NSPasteboard.general.setString(text, forType: .string)
#endif
    }
}
