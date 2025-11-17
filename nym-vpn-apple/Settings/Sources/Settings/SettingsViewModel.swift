import Combine
import SwiftUI
import AppSettings
import AppVersionProvider
import ConfigurationManager
import ConnectionManager
import CredentialsManager
import ExternalLinkManager
import FeatureFlagsManager
import ImpactGenerator
import UIComponents

@MainActor public class SettingsViewModel: SettingsFlowState {
    private let appSettings: AppSettings
    private let configurationManager: ConfigurationManager
    private let connectionManager: ConnectionManager
    private let externalLinkManager: ExternalLinkManager
    private let featureFlagsManager: FeatureFlagsManager
    private let impactGenerator: ImpactGenerator

    @ObservedObject private var credentialsManager: CredentialsManager
    private var cancellables = Set<AnyCancellable>()

    let settingsTitle = "settings".localizedString
#if os(macOS)
    @Binding private var isServing: Bool
#endif
    @Published var isLogoutConfirmationDisplayed = false
    @Published var sections: [SettingsSection] = []
    @Published var accountIdentifier: String?

    var isValidCredentialImported: Bool {
        credentialsManager.isValidCredentialImported
    }

    var logoutDialogConfiguration: ActionDialogConfiguration {
        ActionDialogConfiguration(
            systemIconImageName: "exclamationmark.circle",
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
        appSettings: AppSettings,
        configurationManager: ConfigurationManager,
        connectionManager: ConnectionManager,
        credentialsManager: CredentialsManager,
        externalLinkManager: ExternalLinkManager,
        featureFlagsManager: FeatureFlagsManager,
        impactGenerator: ImpactGenerator
    ) {
        self.appSettings = appSettings
        self.configurationManager = configurationManager
        self.connectionManager = connectionManager
        self.credentialsManager = credentialsManager
        self.externalLinkManager = externalLinkManager
        self.featureFlagsManager = featureFlagsManager
        self.impactGenerator = impactGenerator
        super.init(path: path)
        setup()
    }
#elseif os(macOS)
    public init(
        isServing: Binding<Bool>,
        path: Binding<NavigationPath>,
        appSettings: AppSettings,
        configurationManager: ConfigurationManager,
        connectionManager: ConnectionManager,
        credentialsManager: CredentialsManager,
        externalLinkManager: ExternalLinkManager,
        featureFlagsManager: FeatureFlagsManager,
        impactGenerator: ImpactGenerator
    ) {
        _isServing = isServing
        self.appSettings = appSettings
        self.configurationManager = configurationManager
        self.connectionManager = connectionManager
        self.credentialsManager = credentialsManager
        self.externalLinkManager = externalLinkManager
        self.featureFlagsManager = featureFlagsManager
        self.impactGenerator = impactGenerator
        super.init(path: path)
        setup()
    }
#endif

    func appVersion() -> String {
        AppVersionProvider.appVersion()
    }

    func navigateBack() {
        guard !path.isEmpty else { return }
        impactGenerator.softImpact()
        path.removeLast()
    }

    func navigateToAddCredentialsOrCredential() {
#if os(macOS)
        guard isServing
        else {
            path.append(SettingLink.daemonEnable)
            return
        }
        if credentialsManager.isValidCredentialImported {
            navigateToAccount()
        } else {
            path.append(SettingLink.addCredentials)
        }
#elseif os(iOS)
        impactGenerator.softImpact()
        if credentialsManager.isValidCredentialImported {
            navigateToAccount()
        } else {
            path.append(SettingLink.createAccountWelcome)
        }
#endif
    }

    func navigateToSantasMenu() {
        guard configurationManager.isSantaClaus else { return }
        impactGenerator.impact()
        path.append(SettingLink.santasMenu)
    }
}

private extension SettingsViewModel {
    func navigateToPrivacyAndData() {
        impactGenerator.softImpact()
        path.append(SettingLink.privacyAndData)
    }

    func navigateToAppearance() {
        impactGenerator.softImpact()
        path.append(SettingLink.appearance)
    }

    func navigateToLogs() {
        impactGenerator.softImpact()
        path.append(SettingLink.logs)
    }

    func navigateToSupportAndFeedback() {
        impactGenerator.softImpact()
        path.append(SettingLink.support)
    }

    func navigateToLegal() {
        impactGenerator.softImpact()
        path.append(SettingLink.legal)
    }

    func navigateToAccount() {
        impactGenerator.softImpact()
        path.append(SettingLink.accountAndDevices)
    }

    func navigateToPassphrase() {
        impactGenerator.softImpact()
        path.append(SettingLink.passphrase)
    }

    func navigateToCensorship() {
        impactGenerator.softImpact()
        path.append(SettingLink.censorship)
    }
}

// MARK: - Setup -
private extension SettingsViewModel {
    func setup() {
        setupAppSettingsObservers()
        setupCredentialManagerObservers()
        configureSections()
    }

    func setupAppSettingsObservers() {
        appSettings.$isCredentialImportedPublisher.sink { [weak self] _ in
            self?.configureSections()
        }
        .store(in: &cancellables)
    }

    func setupCredentialManagerObservers() {
        credentialsManager.$accountIdentifier
            .receive(on: DispatchQueue.main)
            .sink { [weak self] newValue in
                MainActor.assumeIsolated {
                    self?.accountIdentifier = newValue
                }
            }
            .store(in: &cancellables)
    }

    func configureSections() {
        Task {
            var newSections = [SettingsSection]()
            if appSettings.isCredentialImported {
                newSections.append(accountSection())
            }
            newSections.append(
                contentsOf: [
                    feedbackSection(),
                    killswitchSection(),
                    appearanceSection(),
                    logsSection(),
                    legalSection()
                ]
            )
            if appSettings.isCredentialImported {
                newSections.append(logoutSection())
            }
            await MainActor.run {
                sections = newSections
            }
        }
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
        var viewModels = [
            SettingsListItemViewModel(
                accessory: .arrow,
                title: "settings.account".localizedString,
                systemImageName: "person.crop.circle",
                action: { [weak self] in
                    Task { @MainActor in
                        self?.navigateToAccount()
                    }
                }
            )
        ]
#if os(iOS)
        viewModels.append(
            SettingsListItemViewModel(
                accessory: .arrow,
                title: "settings.passphrase".localizedString,
                imageName: "key",
                action: { [weak self] in
                    Task { @MainActor in
                        self?.navigateToPassphrase()
                    }
                }
            )
        )
#endif
        return .account(viewModels: viewModels)
    }

    func appearanceSection() -> SettingsSection {
        .theme(
            viewModels: [
                SettingsListItemViewModel(
                    accessory: .arrow,
                    title: "settings.appearance".localizedString,
                    imageName: "appearance",
                    action: { [weak self] in
                        Task { @MainActor in
                            self?.navigateToAppearance()
                        }
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
                        Task { @MainActor in
                            self?.navigateToSupportAndFeedback()
                        }
                    }
                )
            ]
        )
    }

    func killswitchSection() -> SettingsSection {
        var viewModels = [SettingsListItemViewModel]()
        viewModels.append(
            SettingsListItemViewModel(
                accessory: .empty,
                title: "settings.killswitch.title".localizedString,
                subtitle: "settings.killswitch.subtitle".localizedString,
                systemImageName: "power",
                action: {}
            )
        )
#if os(macOS)
        viewModels.append(
            SettingsListItemViewModel(
                accessory: .toggle(viewModel: ToggleViewModel(isOn: appSettings.$isIPv6TrafficEnabled)),
                title: "settings.ipv6.title".localizedString,
                subtitle: "settings.ipv6.subtitle".localizedString,
                systemImageName: "powerplug.portrait",
                action: {}
            )
        )
#endif
        viewModels.append(
            SettingsListItemViewModel(
                accessory: .arrow,
                title: "settings.censorship.title".localizedString,
                imageName: "domain",
                action: { [weak self] in
                    Task { @MainActor in
                        self?.navigateToCensorship()
                    }
                }
            )
        )

        return .killSwitch(viewModels: viewModels)
    }

    func logsSection() -> SettingsSection {
        .logs(
            viewModels: [
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
                    accessory: .arrow,
                    title: "settings.privacyAndData".localizedString,
                    systemImageName: "exclamationmark.shield",
                    action: { [weak self] in
                        Task { @MainActor in
                            self?.navigateToPrivacyAndData()
                        }
                    }
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
        impactGenerator.success()
#if os(iOS)
        UIPasteboard.general.string = text
#elseif os(macOS)
        NSPasteboard.general.prepareForNewContents()
        NSPasteboard.general.setString(text, forType: .string)
#endif
    }
}
