import Combine
import SwiftUI
import AppSettings
import AppVersionProvider
import ConfigurationManager
import ConnectionManager
import ConnectionTypes
import CredentialsManager
import ExternalLinkManager
import FeatureFlagsManager
import ImpactGenerator
#if os(iOS)
import PurchasesManager
#endif
import Routes
import UIComponents
import Theme

@MainActor public class SettingsViewModel: SettingsFlowState {
    public typealias AppSettingsSection = SettingsSection<AppSettingsSectionKind>

    private let appSettings: AppSettings
    private let configurationManager: ConfigurationManager
    private let connectionManager: ConnectionManager
    private let externalLinkManager: ExternalLinkManager
    private let featureFlagsManager: FeatureFlagsManager
    private let impactGenerator: ImpactGenerator
#if os(iOS)
    private let purchasesManager: PurchasesManager
#endif

    @ObservedObject private var credentialsManager: CredentialsManager
    private var cancellables = Set<AnyCancellable>()

    let settingsTitle = "settings".localizedString
#if os(macOS)
    @Binding private var isServing: Bool
#endif
    @Published var sections: [AppSettingsSection] = []
    @Published var accountIdentifier: String?
#if os(macOS)
    var autologinState: AutologinState?
#endif

    var versionTitle: String {
        let base = "\("version".localizedString) \(AppVersionProvider.realAppVersion())"
        let env = configurationManager.currentEnvString
        guard env != Env.mainnet.rawValue else { return base }
        return "\(base) - \(env)"
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
        impactGenerator: ImpactGenerator,
        purchasesManager: PurchasesManager
    ) {
        self.appSettings = appSettings
        self.configurationManager = configurationManager
        self.connectionManager = connectionManager
        self.credentialsManager = credentialsManager
        self.externalLinkManager = externalLinkManager
        self.featureFlagsManager = featureFlagsManager
        self.impactGenerator = impactGenerator
        self.purchasesManager = purchasesManager
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

    func navigateToSantasMenu() {
#if SANTA
        guard configurationManager.isSantaClaus else { return }
        impactGenerator.impact()
        path.append(SettingLink.santasMenu)
#endif
    }

    /// Use to reload sections and acc renewal info
    func reloadSections() {
        configureSections()
    }

    func updateAccountSectionOnly() {
        guard appSettings.isCredentialImported else {
            sections.removeAll { $0.kind == .account }
            return
        }
        let updated = accountSection()
        if let index = sections.firstIndex(where: { $0.kind == .account }) {
            sections[index] = updated
        } else {
            sections.insert(updated, at: 0)
        }
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

    func navigateToPlanPurchase() {
        impactGenerator.softImpact()
#if os(iOS)
        path.append(SettingLink.generatePassphrase(displayPurchaseView: true))
#elseif os(macOS)
        autologinState?.start(kind: .autologinRenew, using: credentialsManager)
#endif
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

    func navigateToDns() {
        impactGenerator.softImpact()
        path.append(SettingLink.dns)
    }

    func navigateToMixnetTuning() {
        impactGenerator.softImpact()
        path.append(SettingLink.mixnetTuning)
    }

#if os(macOS)
    func navigateToGeoExclusion() {
        impactGenerator.softImpact()
        path.append(SettingLink.geoExclusion)
    }

    func navigateToSplitTunneling() {
        impactGenerator.softImpact()
        path.append(SettingLink.splitTunnel)
    }
#endif

    func navigateToCensorship() {
        impactGenerator.softImpact()
        path.append(SettingLink.censorship)
    }

    func navigateToNotifications() {
        impactGenerator.softImpact()
        path.append(SettingLink.notifications)
    }
}

// MARK: - Setup -
private extension SettingsViewModel {
    func setup() {
        setupAppSettingsObservers()
        setupCredentialManagerObservers()
        reloadSections()
    }

    func setupAppSettingsObservers() {
        appSettings.$isCredentialImportedPublisher.sink { [weak self] _ in
            self?.reloadSections()
        }
        .store(in: &cancellables)

        Publishers.Merge3(
            appSettings.$isAdBlockerEnabledPublisher,
            appSettings.$isIPv6TrafficEnabledPublisher,
            appSettings.$isLanBypassEnabledPublisher
        )
        .receive(on: DispatchQueue.main)
        .sink { [weak self] _ in
            self?.objectWillChange.send()
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

        credentialsManager.$accountSummary
            .receive(on: DispatchQueue.main)
            .sink { [weak self] _ in
                MainActor.assumeIsolated {
                    self?.updateAccountSectionOnly()
                }
            }
            .store(in: &cancellables)
    }

    /// Configures sections, to reload all the content - use reloadSections
    func configureSections() {
        Task {
            var newSections = [AppSettingsSection]()
            if appSettings.isCredentialImported {
                newSections.append(accountSection())
            }
            newSections.append(
                contentsOf: [
                    feedbackSection(),
                    notificationsSection(),
                    killswitchSection(),
                    appearanceSection(),
                    privacyAndDataSection(),
                    legalSection(),
                    systemStatusSection()
                ]
            )
            await MainActor.run {
                sections = newSections
            }
        }
    }
}

// MARK: - Helpers -
private extension SettingsViewModel {
    func isAutoRenewEnabled(accountSummary: AccountSummary) -> Bool {
#if os(iOS)
        purchasesManager.isAutoRenewEnabled || accountSummary.isAutoRenewEnabled
#elseif os(macOS)
        accountSummary.isAutoRenewEnabled
#endif
    }

    static func noActivePlanChoosePlanSubtitle() -> AttributedString {
        var first = AttributedString("noActivePlan".localizedString)
        first.foregroundColor = Color.Nym.error
        var second = AttributedString("\n\( "purchasePlan.chooseMyPlan".localizedString)")
        second.foregroundColor = Color.Nym.primary
        return first + second
    }
}

extension SettingsViewModel {
    enum NilSummaryAccountCopy: Equatable {
        case requestingZkNyms
        case unreachable
        case noActivePlan
    }

    static func nilSummaryAccountCopy(
        lastFetchFailed: Bool,
        isRegistrationInFlight: Bool
    ) -> NilSummaryAccountCopy {
        if isRegistrationInFlight { return .requestingZkNyms }
        if lastFetchFailed { return .unreachable }
        return .noActivePlan
    }
}

// MARK: - Sections -
private extension SettingsViewModel {
    func accountSection() -> AppSettingsSection {
        let subtitle: AttributedString
        if let accountSummary = credentialsManager.accountSummary {
            if let planText = accountSummary.planValidUntilAttributedString {
                if accountSummary.isActive,
                   isAutoRenewEnabled(accountSummary: accountSummary),
                   !accountSummary.isExpiringSoon,
                   !accountSummary.isExpiringWarning {
                    var second = AttributedString("* \("autoRenews".localizedString)")
                    second.foregroundColor = Color.Nym.textSecondary
                    subtitle = planText + AttributedString("\n") + second
                } else {
                    subtitle = planText
                }
            } else if accountSummary.subscription?.status == .pending {
                var confirmingPayment = AttributedString("confirmingPayment".localizedString)
                confirmingPayment.foregroundColor = Color.Nym.error
                subtitle = confirmingPayment
            } else {
                subtitle = Self.noActivePlanChoosePlanSubtitle()
            }
        } else {
            switch Self.nilSummaryAccountCopy(
                lastFetchFailed: credentialsManager.accountSummaryLastFetchFailed,
                isRegistrationInFlight: credentialsManager.isAccountRegistrationInFlight
            ) {
            case .requestingZkNyms:
                subtitle = AttributedString("requestingZkNyms".localizedString)
            case .unreachable:
                subtitle = AttributedString("home.accountUnreachable".localizedString)
            case .noActivePlan:
                subtitle = Self.noActivePlanChoosePlanSubtitle()
            }
        }

        var viewModels = [
            SettingsListItemViewModel(
                accessory: .arrow,
                title: "settings.account".localizedString,
                attributtedSubtitle: subtitle,
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

    func notificationsSection() -> AppSettingsSection {
        AppSettingsSection(
            kind: .notifications,
            viewModels: [
                SettingsListItemViewModel(
                    accessory: .arrow,
                    title: "settings.notifications.title".localizedString,
                    systemImageName: "bell",
                    action: { [weak self] in
                        Task { @MainActor in
                            self?.navigateToNotifications()
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

    // swiftlint:disable:next function_body_length
    func killswitchSection() -> AppSettingsSection {
        var viewModels = [SettingsListItemViewModel]()
        viewModels.append(
            SettingsListItemViewModel(
                accessory: .empty,
                title: "settings.killswitch.title".localizedString,
                subtitle: "settings.killswitch.subtitle".localizedString,
                systemImageName: "power",
                isHoveredHighlightDisabled: true,
                action: {}
            )
        )
        let adBlockSubtitle = appSettings.isAdBlockerEnabled
        ? "settings.adblock.subtitle.on".localizedString
        : "settings.adblock.subtitle.off".localizedString

        let adBlockViewModel = SettingsListItemViewModel(
            accessory: .toggle(
                isOn: Binding(
                    get: { [appSettings] in appSettings.isAdBlockerEnabled },
                    set: { [connectionManager] newValue in connectionManager.setAdBlocking(newValue) }
                )
            ),
            title: "settings.adblock.title".localizedString,
            subtitle: adBlockSubtitle,
            systemImageName: "exclamationmark.shield",
            action: {}
        )
        appSettings.$isAdBlockerEnabledPublisher
            .dropFirst()
            .receive(on: DispatchQueue.main)
            .sink { isOn in
                adBlockViewModel.subtitle = AttributedString(
                    isOn
                        ? "settings.adblock.subtitle.on".localizedString
                        : "settings.adblock.subtitle.off".localizedString
                )
            }
            .store(in: &cancellables)
        viewModels.append(adBlockViewModel)
#if os(macOS)
        viewModels.append(
            SettingsListItemViewModel(
                accessory: .toggle(
                    isOn: Binding(
                        get: { [appSettings] in appSettings.isIPv6TrafficEnabled },
                        set: { [connectionManager] newValue in connectionManager.setIPv6TrafficEnabled(newValue) }
                    )
                ),
                title: "settings.ipv6.title".localizedString,
                subtitle: "settings.ipv6.subtitle".localizedString,
                imageName: "powerplug.portrait",
                action: {}
            )
        )
#endif
        viewModels.append(
            SettingsListItemViewModel(
                accessory: .toggle(
                    isOn: Binding(
                        get: { [appSettings] in appSettings.isLanBypassEnabled },
                        set: { [connectionManager] newValue in connectionManager.setLanBypassEnabled(newValue) }
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
                title: "settings.geoExclusion".localizedString,
                subtitle: "settings.geoExclusion.subtitle".localizedString,
                imageName: "pin",
                badge: "general.beta".localizedString,
                action: { [weak self] in
                    self?.navigateToGeoExclusion()
                }
            )
        )
        viewModels.append(
            SettingsListItemViewModel(
                accessory: .arrow,
                title: "settings.splitTunnel".localizedString,
                imageName: "arrow.trianglehead.branch",
                action: { [weak self] in
                    self?.navigateToSplitTunneling()
                }
            )
        )
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
                title: "settings.dns.title".localizedString,
                imageName: "dns",
                action: { [weak self] in
                    Task { @MainActor in
                        self?.navigateToDns()
                    }
                }
            )
        )
        viewModels.append(
            SettingsListItemViewModel(
                accessory: .arrow,
                title: "settings.mixnetTuning.title".localizedString,
                subtitle: "settings.mixnetTuning.subtitle".localizedString,
                systemImageName: "eye.slash",
                action: { [weak self] in
                    self?.navigateToMixnetTuning()
                }
            )
        )
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

    func privacyAndDataSection() -> AppSettingsSection {
        let viewModels = [
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
