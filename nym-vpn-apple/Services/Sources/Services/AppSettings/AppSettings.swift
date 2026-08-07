import Foundation
import SwiftUI
import Constants
import ConnectionTypes

@MainActor public final class AppSettings: ObservableObject {
    public static let shared = AppSettings()

    // Duplicated from MockMode — importing ConnectionManager here would cycle.
    static var isMockMode: Bool {
        #if MOCK_MODE
        return true
        #elseif DEBUG
        return ProcessInfo.processInfo.arguments.contains("-MOCK_MODE")
            || ProcessInfo.processInfo.arguments.contains("MOCK_MODE")
        #else
        return false
        #endif
    }

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
    public var currentAppearance: AppSetting.Appearance = .automatic {
        didSet {
            let appearance = currentAppearance.nsAppearance
            NSApp.appearance = appearance
            for window in NSApp.windows {
                window.appearance = appearance
            }
        }
    }

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

    @AppStorage(AppSettingKey.didCompleteFirstLaunch.rawValue)
    public var didCompleteFirstLaunch = false

    // Technical opt ins
    @AppStorage(AppSettingKey.welcomeScreenDidDisplay.rawValue)
    public var welcomeScreenDidDisplay = false

    @AppStorage(AppSettingKey.onboardingDidDisplay.rawValue)
    public var onboardingDidDisplay = false

    @AppStorage(AppSettingKey.connectionConfig.rawValue)
    public var connectionConfig: String?

    @AppStorage(AppSettingKey.countryStore.rawValue)
    public var countryStore: String?

    @AppStorage(AppSettingKey.gatewayStore.rawValue)
    public var gatewayStore: String?

    @AppStorage(AppSettingKey.currentEnv.rawValue, store: UserDefaults(suiteName: Constants.groupID.rawValue))
    public var currentEnv: String = "mainnet"

    @AppStorage(AppSettingKey.accountToken.rawValue)
    public var accountToken: String?

    @AppStorage(AppSettingKey.passphraseStored.rawValue)
    public var isPassphraseStored: Bool = false {
        didSet { isPassphraseStoredPublisher = isPassphraseStored }
    }

    @AppStorage(AppSettingKey.ipv6TrafficIsEnabled.rawValue)
    public var isIPv6TrafficEnabled = true {
        didSet { isIPv6TrafficEnabledPublisher = isIPv6TrafficEnabled }
    }

    @AppStorage(AppSettingKey.lanBypass.rawValue)
    public var isLanBypassEnabled = false {
        didSet { isLanBypassEnabledPublisher = isLanBypassEnabled }
    }

    @AppStorage(AppSettingKey.isAdBlockerEnabled.rawValue)
    public var isAdBlockerEnabled = false {
        didSet { isAdBlockerEnabledPublisher = isAdBlockerEnabled}
    }

    @AppStorage(AppSettingKey.serverFamilyReminders.rawValue)
    public var serverFamilyRemindersEnabled = true {
        didSet { serverFamilyRemindersEnabledPublisher = serverFamilyRemindersEnabled }
    }

#if os(macOS)
    @AppStorage(AppSettingKey.statistics.rawValue)
    public var isStatisticsEnabled = true
#else
    public var isStatisticsEnabled = false
#endif

    @AppStorage(AppSettingKey.statisticsConnectionCount.rawValue)
    public var statisticsConnectionCount = 0

    @AppStorage(AppSettingKey.quic.rawValue)
    public var isQuicEnabled = false {
        didSet { isQuicEnabledPublisher = isQuicEnabled }
    }

    @AppStorage(AppSettingKey.stealthApi.rawValue)
    public var isStealthApiEnabled = false {
        didSet { isStealthApiEnabledPublisher = isStealthApiEnabled }
    }

    @AppStorage(AppSettingKey.customDnsIsEnabled.rawValue)
    public var isCustomDnsEnabled = false {
        didSet { isCustomDnsEnabledPublisher = isCustomDnsEnabled }
    }

    @AppStorage(AppSettingKey.isMixnetTuningEnabled.rawValue)
    public var isMixnetTuningEnabled = false {
        didSet {
            isMixnetTuningEnabledPublisher = isMixnetTuningEnabled
        }
    }

    @AppStorage(AppSettingKey.customDns.rawValue)
    public var customDns: [String] = []

    @AppStorage(AppSettingKey.isDebugLogsOn.rawValue, store: UserDefaults(suiteName: Constants.groupID.rawValue))
    public var isDebugLogsOn = false

    @AppStorage(AppSettingKey.expiryWarningDismissedAt.rawValue)
    public var expiryWarningDismissedAt: Double = 0

    @AppStorage(AppSettingKey.expiryWarningSoonDismissedAt.rawValue)
    public var expirySoonDismissedAt: Double = 0

    @AppStorage(AppSettingKey.accountSummaryCache.rawValue)
    public var accountSummaryCache: String?

    @AppStorage(AppSettingKey.accountSummaryLastFetchedAt.rawValue)
    public var accountSummaryLastFetchedAt: Double = 0

    @AppStorage(AppSettingKey.oneClickDisplayMode.rawValue)
    public var oneClickDisplayModeRaw: String = "powerUser"

    public var accountSummary: AccountSummary? {
        get {
            guard let json = accountSummaryCache,
                  let data = json.data(using: .utf8)
            else {
                return nil
            }
            return try? JSONDecoder().decode(AccountSummary.self, from: data)
        }
        set {
            guard let newValue,
                  let data = try? JSONEncoder().encode(newValue),
                  let json = String(data: data, encoding: .utf8)
            else {
                accountSummaryCache = nil
                accountSummaryLastFetchedAt = 0
                return
            }
            accountSummaryCache = json
            accountSummaryLastFetchedAt = Date().timeIntervalSince1970
        }
    }

    // Observed values for view models
    @Published public var isErrorReportingOnPublisher: Bool
    @Published public var isCredentialImportedPublisher: Bool
    @Published public var isQuicEnabledPublisher: Bool
    @Published public var isStealthApiEnabledPublisher: Bool
    @Published public var isCustomDnsEnabledPublisher: Bool
    @Published public var customDnsPublisher: [String]
    @Published public var isIPv6TrafficEnabledPublisher: Bool
    @Published public var isLanBypassEnabledPublisher: Bool
    @Published public var isMixnetTuningEnabledPublisher: Bool
    @Published public var isAdBlockerEnabledPublisher: Bool
    @Published public var isPassphraseStoredPublisher: Bool
    @Published public var serverFamilyRemindersEnabledPublisher: Bool

    // Init ensures the *Publisher mirrors stored values* on launch.
    private init() {
        self.isErrorReportingOnPublisher = false
        self.isCredentialImportedPublisher = false
        self.isQuicEnabledPublisher = false
        self.isStealthApiEnabledPublisher = false
        self.isCustomDnsEnabledPublisher = false
        self.customDnsPublisher = []
        self.isIPv6TrafficEnabledPublisher = true
        self.isLanBypassEnabledPublisher = false
        self.isMixnetTuningEnabledPublisher = false
        isAdBlockerEnabledPublisher = false
        isPassphraseStoredPublisher = false
        serverFamilyRemindersEnabledPublisher = true

        // Seed a signed-in mock session so UI tests start on the home screen.
        if Self.isMockMode {
            isCredentialImported = true
            welcomeScreenDidDisplay = true
            onboardingDidDisplay = true
        }

        self.isErrorReportingOnPublisher = self.isErrorReportingOn
        self.isCredentialImportedPublisher = self.isCredentialImported
        self.isQuicEnabledPublisher = self.isQuicEnabled
        self.isStealthApiEnabledPublisher = self.isStealthApiEnabled
        self.isCustomDnsEnabledPublisher = self.isCustomDnsEnabled
        self.customDnsPublisher = self.customDns
        self.isIPv6TrafficEnabledPublisher = self.isIPv6TrafficEnabled
        self.isLanBypassEnabledPublisher = self.isLanBypassEnabled
        self.isMixnetTuningEnabledPublisher = self.isMixnetTuningEnabled
        self.isAdBlockerEnabledPublisher = self.isAdBlockerEnabled
        self.isPassphraseStoredPublisher = self.isPassphraseStored
        self.serverFamilyRemindersEnabledPublisher = self.serverFamilyRemindersEnabled
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
    case didCompleteFirstLaunch
    case welcomeScreenDidDisplay
    case onboardingDidDisplay
    case lastConnectionIntent
    case currentEnv
    case countryStore
    case gatewayStore
    case accountToken
    case accountTokensByEnv
    case ipv6TrafficIsEnabled
    case statistics
    case statisticsConnectionCount
    case quic
    case stealthApi
    case lanBypass
    case passphraseStored
    case connectionConfig
    case customDnsIsEnabled
    case customDns
    case isMixnetTuningEnabled
    case isAdBlockerEnabled
    case isDebugLogsOn
    case expiryWarningDismissedAt
    case expiryWarningSoonDismissedAt
    case accountSummaryCache
    case accountSummaryLastFetchedAt
    case oneClickDisplayMode
    case serverFamilyReminders
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
