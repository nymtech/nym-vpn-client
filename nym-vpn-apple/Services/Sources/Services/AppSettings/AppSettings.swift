import SwiftUI
import Constants
import CountriesManagerTypes

@MainActor public final class AppSettings: ObservableObject {
    public static let shared = AppSettings()

#if os(iOS)
    @AppStorage(AppSettingKey.currentAppearance.rawValue)
    public var currentAppearance: AppSetting.Appearance = .automatic {
        didSet {
            guard let keyWindow = AppSettings.keyWindow else { return }
            keyWindow.rootViewController?.overrideUserInterfaceStyle = currentAppearance.userInterfaceStyle
        }
    }
#else
    @AppStorage(AppSettingKey.currentAppearance.rawValue)
    public var currentAppearance: AppSetting.Appearance = .light

    @AppStorage(AppSettingKey.appMode.rawValue)
    public var appMode: AppSetting.AppMode = .both
#endif

    @AppStorage(AppSettingKey.errorReporting.rawValue)
    public var isErrorReportingOn = false {
        didSet { isErrorReportingOnPublisher = isErrorReportingOn }
    }

    @AppStorage(AppSettingKey.credenitalExists.rawValue)
    public var isCredentialImported = false {
        didSet { isCredentialImportedPublisher = isCredentialImported }
    }

    @AppStorage(AppSettingKey.smallScreen.rawValue)
    public var isSmallScreen = false

    // Technical opt ins
    @AppStorage(AppSettingKey.welcomeScreenDidDisplay.rawValue)
    public var welcomeScreenDidDisplay = false

    @AppStorage(AppSettingKey.onboardingDidDisplay.rawValue)
    public var onboardingDidDisplay = false

    @AppStorage(AppSettingKey.entryGateway.rawValue)
    public var entryGateway: String?

    @AppStorage(AppSettingKey.exitRouter.rawValue)
    public var exitRouter: String?

    @AppStorage(AppSettingKey.connectionConfig.rawValue)
    public var connectionConfig: String?

    @AppStorage(AppSettingKey.connectionType.rawValue)
    public var connectionType: Int?

    @AppStorage(AppSettingKey.countryStore.rawValue)
    public var countryStore: String?

    @AppStorage(AppSettingKey.gatewayStore.rawValue)
    public var gatewayStore: String?

    @AppStorage(AppSettingKey.currentEnv.rawValue, store: UserDefaults(suiteName: Constants.groupID.rawValue))
    public var currentEnv: String = "mainnet"

    @AppStorage(AppSettingKey.accountToken.rawValue)
    public var accountToken: String?

    @AppStorage(AppSettingKey.passphraseStored.rawValue)
    public var isPassphraseStored: Bool = false

    @AppStorage(AppSettingKey.ipv6TrafficIsEnabled.rawValue)
    public var isIPv6TrafficEnabled = true {
        didSet { isIPv6TrafficEnabledPublisher = isIPv6TrafficEnabled }
    }

    @AppStorage(AppSettingKey.lanBypass.rawValue)
    public var isLanBypassEnabled = false {
        didSet { isLanBypassEnabledPublisher = isLanBypassEnabled }
    }

    @AppStorage(AppSettingKey.statistics.rawValue)
    public var isStatisticsEnabled = true

    @AppStorage(AppSettingKey.statisticsConnectionCount.rawValue)
    public var statisticsConnectionCount = 0

    @AppStorage(AppSettingKey.quic.rawValue)
    public var isQuicEnabled = false {
        didSet { isQuicEnabledPublisher = isQuicEnabled }
    }
    @AppStorage(AppSettingKey.shouldReconnect.rawValue)
    public var shouldReconnect = false {
        didSet { shouldReconnectPublisher = shouldReconnect }
    }

    @AppStorage(AppSettingKey.customDnsIsEnabled.rawValue)
    public var isCustomDnsEnabled = false {
        didSet { isCustomDnsEnabledPublisher = isCustomDnsEnabled }
    }

    @AppStorage(AppSettingKey.customDns.rawValue)
    public var customDns: [String] = []

    // Observed values for view models
    @Published public var isErrorReportingOnPublisher: Bool
    @Published public var isCredentialImportedPublisher: Bool
    @Published public var isQuicEnabledPublisher: Bool
    @Published public var shouldReconnectPublisher: Bool
    @Published public var isCustomDnsEnabledPublisher: Bool
    @Published public var customDnsPublisher: [String]
    @Published public var isIPv6TrafficEnabledPublisher: Bool
    @Published public var isLanBypassEnabledPublisher: Bool

    // Init ensures the *Publisher mirrors stored values* on launch.
    private init() {
        self.isErrorReportingOnPublisher = false
        self.isCredentialImportedPublisher = false
        self.isQuicEnabledPublisher = false
        self.shouldReconnectPublisher = false
        self.isCustomDnsEnabledPublisher = false
        self.customDnsPublisher = []
        self.isIPv6TrafficEnabledPublisher = true
        self.isLanBypassEnabledPublisher = false

        self.isErrorReportingOnPublisher = self.isErrorReportingOn
        self.isCredentialImportedPublisher = self.isCredentialImported
        self.isQuicEnabledPublisher = self.isQuicEnabled
        self.shouldReconnectPublisher = self.shouldReconnect
        self.isCustomDnsEnabledPublisher = self.isCustomDnsEnabled
        self.customDnsPublisher = self.customDns
        self.isIPv6TrafficEnabledPublisher = self.isIPv6TrafficEnabled
        self.isLanBypassEnabledPublisher = self.isLanBypassEnabled
    }

    public func resetUserDefaults() {
        let defaults = UserDefaults.standard
        let dictionary = defaults.dictionaryRepresentation()
        dictionary.keys.forEach { key in
            defaults.removeObject(forKey: key)
        }
    }
}

#if os(iOS)
private extension AppSettings {
    static var keyWindow: UIWindow? {
        UIApplication.shared.connectedScenes
            .compactMap { $0 as? UIWindowScene }
            .flatMap { $0.windows }
            .first(where: { $0.isKeyWindow })
    }
}
#endif

public enum AppSettingKey: String {
    case currentAppearance
    case appMode
    case errorReporting
    case credenitalExists
    case smallScreen
    case welcomeScreenDidDisplay
    case onboardingDidDisplay
    case entryGateway
    case exitRouter
    case connectionType
    case lastConnectionIntent
    case currentEnv
    case countryStore
    case gatewayStore
    case accountToken
    case ipv6TrafficIsEnabled
    case statistics
    case statisticsConnectionCount
    case quic
    case lanBypass
    case shouldReconnect
    case passphraseStored
    case connectionConfig
    case customDnsIsEnabled
    case customDns
}

extension Array: @retroactive RawRepresentable where Element: Codable {
    public init?(rawValue: String) {
        guard
            let data = rawValue.data(using: .utf8),
            let result = try? JSONDecoder().decode([Element].self, from: data)
        else { return nil }
        self = result
    }

    public var rawValue: String {
        guard
            let data = try? JSONEncoder().encode(self),
            let result = String(data: data, encoding: .utf8)
        else { return "" }
        return result
    }
}
