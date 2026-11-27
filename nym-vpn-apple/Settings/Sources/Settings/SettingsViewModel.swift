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

    let appSettings: AppSettings
    let configurationManager: ConfigurationManager
    let connectionManager: ConnectionManager
    private let externalLinkManager: ExternalLinkManager
    private let featureFlagsManager: FeatureFlagsManager
    let impactGenerator: ImpactGenerator
#if os(iOS)
    let purchasesManager: PurchasesManager
#endif

    @ObservedObject var credentialsManager: CredentialsManager
    var cancellables = Set<AnyCancellable>()

    let settingsTitle = "settings".localizedString
#if os(macOS)
    @Binding private var isServing: Bool
#endif
    @Published var sections: [AppSettingsSection] = []
    @Published var accountIdentifier: String?
    @Published var isCodeEntryPresented = false
    @Published var isLogoutConfirmationDisplayed = false
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

    var isSignedIn: Bool {
        appSettings.isCredentialImported
    }

    func navigateBack() {
        guard !path.isEmpty else { return }
        impactGenerator.softImpact()
        path.removeLast()
    }

    func navigateToRoot() {
        path = NavigationPath()
    }

    func navigate(to link: SettingLink) {
        impactGenerator.softImpact()
        path.append(link)
    }

    func navigateToSantasMenu() {
#if SANTA
        guard configurationManager.isSantaClaus else { return }
        impactGenerator.impact()
        path.append(SettingLink.santasMenu)
#endif
    }

    /// Back to the signed-out home screen, where Get started lives.
    func getStarted() {
        impactGenerator.softImpact()
        navigateToRoot()
    }

    /// Use to reload sections and acc renewal info
    func reloadSections() {
        configureSections()
    }

    func updateAccountSectionOnly() {
        let updated = accountSection()
        if let index = sections.firstIndex(where: { $0.kind == .account }) {
            sections[index] = updated
        } else {
            sections.insert(updated, at: 0)
        }
    }

    func blocks(for kind: AppSettingsSectionKind) -> [SettingsBlock] {
        switch kind {
        case .account:
            isSignedIn ? [] : [.getStarted]
        case .footer:
            [.appVersion(versionTitle)]
        case .protection, .connection, .routing, .app, .helpAndData:
            []
        }
    }

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
            let newSections = [
                accountSection(),
                protectionSection(),
                connectionSection(),
                routingSection(),
                appSection(),
                helpAndDataSection(),
                footerSection()
            ]
            .compactMap { $0 }
            await MainActor.run {
                sections = newSections
            }
        }
    }
}
