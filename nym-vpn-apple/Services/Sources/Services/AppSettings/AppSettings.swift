import SwiftUI
import Constants
import CountriesManagerTypes

public final class AppSettings: ObservableObject {
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
    #endif
    @AppStorage(AppSettingKey.entryLocation.rawValue)
    public var isEntryLocationSelectionOn = false {
        didSet {
            isEntryLocationSelectionOnPublisher = isEntryLocationSelectionOn
        }
    }
    @AppStorage(AppSettingKey.errorReporting.rawValue)
    public var isErrorReportingOn = false {
        didSet {
            Task { @MainActor in
                isErrorReportingOnPublisher = isErrorReportingOn
            }
        }
    }
    @AppStorage(AppSettingKey.credenitalExists.rawValue)
    public var isCredentialImported = false
    @AppStorage(AppSettingKey.credentialExpiryDate.rawValue)
    public var credentialExpiryDate: Date?
    @AppStorage(AppSettingKey.credentialStartDate.rawValue)
    public var credentialStartDate: Date?
    @AppStorage(AppSettingKey.smallScreen.rawValue)
    public var isSmallScreen = false
    @AppStorage(AppSettingKey.welcomeScreenDidDisplay.rawValue)
    public var welcomeScreenDidDisplay = false
    @AppStorage(AppSettingKey.entryCountry.rawValue)
    public var entryCountryCode = ""
    @AppStorage(AppSettingKey.exitCountry.rawValue)
    public var exitCountryCode = ""
    @AppStorage(AppSettingKey.connectionType.rawValue)
    public var connectionType: Int?
    @AppStorage(AppSettingKey.lastConnectionIntent.rawValue)
    public var lastConnectionIntent: String?
    @AppStorage(AppSettingKey.countryStore.rawValue)
    public var countryStore: String?

    @AppStorage(
        AppSettingKey.currentEnv.rawValue,
        store: UserDefaults(suiteName: Constants.groupID.rawValue)
    )
    public var currentEnv: String = "mainnet"

    // Observed values for view models
    @Published public var isEntryLocationSelectionOnPublisher = false
    @Published public var isErrorReportingOnPublisher = false
}

#if os(iOS)
private extension AppSettings {
    static var keyWindow: UIWindow? {
        guard let window = UIApplication.shared.connectedScenes
            .compactMap({ $0 as? UIWindowScene })
            .flatMap({ $0.windows })
            .first(where: { $0.isKeyWindow })
        else {
            return nil
        }
        return window
    }
}
#endif

enum AppSettingKey: String {
    case currentAppearance
    case entryLocation
    case errorReporting
    case credenitalExists
    case credentialExpiryDate
    case credentialStartDate
    case smallScreen
    case welcomeScreenDidDisplay
    case entryCountry
    case exitCountry
    case connectionType
    case lastConnectionIntent
    case currentEnv
    case countryStore
}
