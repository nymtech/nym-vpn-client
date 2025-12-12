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
    public typealias AppSettingsSection = SettingsSection<AppSettingsSectionKind>

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
    @Published var isLogoutLoading = false
    @Published var sections: [AppSettingsSection] = []
    @Published var accountIdentifier: String?

    var isValidCredentialImported: Bool {
        credentialsManager.isValidCredentialImported
    }

    var logoutDialogConfiguration: ActionDialogConfiguration {
        ActionDialogConfiguration(
            systemIconImageName: "rectangle.portrait.and.arrow.right",
            titleLocalizedString: "settings.logoutTitle".localizedString,
            subtitleLocalizedString: "settings.logoutSubtitle".localizedString,
            yesLocalizedString: "settings.logout".localizedString,
            noLocalizedString: "cancel".localizedString,
            isYesDestructive: true,
            yesAction: { [weak self] in
                self?.isLogoutLoading = true
                Task {
                    await self?.logout()
                    try? await Task.sleep(for: .seconds(1))
                    Task { @MainActor in
                        self?.isLogoutConfirmationDisplayed = false
                        self?.isLogoutLoading = false
                    }
                }
            },
            loadingText: "settings.loggingOut".localizedString,
            shouldCloseAfterYesAction: false,
            verticalButtonsLayout: true
        )
    }

    var versionTitle: String {
        "\("version".localizedString) \(AppVersionProvider.appVersion()) (\(AppVersionProvider.libVersion))"
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

    func navigateToSystemStatus() {
        impactGenerator.softImpact()
        path.append(SettingLink.systemStatus)
    }

    func navigateToAccount() {
        impactGenerator.softImpact()
        path.append(SettingLink.accountAndDevices)
    }

    func navigateToPassphrase() {
        impactGenerator.softImpact()
        path.append(SettingLink.passphrase)
    }

    #if os(macOS)
    func navigateToProxy() {
        impactGenerator.softImpact()
        path.append(SettingLink.proxy)
    }
    #endif

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
            var newSections = [AppSettingsSection]()
            if appSettings.isCredentialImported {
                newSections.append(accountSection())
            }
            newSections.append(
                contentsOf: [
                    feedbackSection(),
                    killswitchSection(),
                    appearanceSection(),
                    logsSection(),
                    legalSection(),
                    systemStatusSection()
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
    func accountSection() -> AppSettingsSection {
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
        return AppSettingsSection(kind: .account, viewModels: viewModels)
    }

    func appearanceSection() -> AppSettingsSection {
        AppSettingsSection(
            kind: .theme,
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

    func feedbackSection() -> AppSettingsSection {
        AppSettingsSection(
            kind: .feedback,
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

    func killswitchSection() -> AppSettingsSection {
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
                accessory: .toggle(
                    viewModel: ToggleViewModel(
                        isOn: appSettings.$isIPv6TrafficEnabled,
                        action: { [weak self] isOn in
                            self?.appSettings.isIPv6TrafficEnabled = isOn
                        }
                    )
                ),
                title: "settings.ipv6.title".localizedString,
                subtitle: "settings.ipv6.subtitle".localizedString,
                systemImageName: "powerplug.portrait",
                action: {}
            )
        )
#endif
        viewModels.append(
            SettingsListItemViewModel(
                accessory: .toggle(
                    viewModel: ToggleViewModel(
                        isOn: appSettings.$isLanBypassEnabled,
                        action: { [weak self] isOn in
                            self?.appSettings.isLanBypassEnabled = isOn
                        }
                    )
                ),
                title: "settings.lanBypass.title".localizedString,
                subtitle: "settings.lanBypass.subtitle".localizedString,
                imageName: "lan",
                action: {}
            )
        )
#if os(macOS)
        viewModels.append(
            SettingsListItemViewModel(
                accessory: .arrow,
                title: "settings.proxy.title".localizedString,
                subtitle: "settings.proxy.subtitle".localizedString,
                imageName: "proxy",
                action: { [weak self] in
                    Task { @MainActor in
                        self?.navigateToProxy()
                    }
                }
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

        return AppSettingsSection(kind: .killSwitch, viewModels: viewModels)
    }

    func logsSection() -> AppSettingsSection {
        let viewModels = [
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
        return AppSettingsSection(kind: .logs, viewModels: viewModels)
    }

    func legalSection() -> AppSettingsSection {
        let viewModels = [
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
        return AppSettingsSection(kind: .legal, viewModels: viewModels)
    }

    func systemStatusSection() -> AppSettingsSection {
        let viewModels = [
            SettingsListItemViewModel(
                accessory: .arrow,
                title: "settings.systemStatus".localizedString,
                action: { [weak self] in
                    self?.navigateToSystemStatus()
                }
            )
        ]
        return AppSettingsSection(kind: .systemStatus, viewModels: viewModels)
    }

    func logoutSection() -> AppSettingsSection {
        let viewModels = [
            SettingsListItemViewModel(
                accessory: .empty,
                title: "settings.logout".localizedString,
                type: .destructive,
                action: { [weak self] in
                    self?.isLogoutConfirmationDisplayed = true
                }
            )
        ]
        return AppSettingsSection(kind: .logout, viewModels: viewModels)
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
