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

    @AppStorage(AppSettingKey.appMode.rawValue)
    public var appMode: AppSetting.AppMode = .both
#endif

    @AppStorage(AppSettingKey.errorReporting.rawValue)
    public var isErrorReportingOn = false {
        didSet {
            Task { @MainActor in
                isErrorReportingOnPublisher = isErrorReportingOn
            }
        }
    }
    @AppStorage(AppSettingKey.credenitalExists.rawValue)
    public var isCredentialImported = false {
        didSet {
            isCredentialImportedPublisher = isCredentialImported
        }
    }
    @AppStorage(AppSettingKey.smallScreen.rawValue)
    public var isSmallScreen = false
    @AppStorage(AppSettingKey.welcomeScreenDidDisplay.rawValue)
    public var welcomeScreenDidDisplay = false

    // TODO: remove after migration. Introduced in 1.6.0
    @AppStorage(AppSettingKey.entryCountry.rawValue)
    public var entryCountryCode = ""
    // TODO: remove after migration. Introduced in 1.6.0
    @AppStorage(AppSettingKey.exitCountry.rawValue)
    public var exitCountryCode = ""

    @AppStorage(AppSettingKey.entryGateway.rawValue)
    public var entryGateway: String?
    @AppStorage(AppSettingKey.exitRouter.rawValue)
    public var exitRouter: String?

    @AppStorage(AppSettingKey.connectionType.rawValue)
    public var connectionType: Int?
    @AppStorage(AppSettingKey.lastConnectionIntent.rawValue)
    public var lastConnectionIntent: String?
    @AppStorage(AppSettingKey.countryStore.rawValue)
    public var countryStore: String?
    @AppStorage(AppSettingKey.gatewayStore.rawValue)
    public var gatewayStore: String?

    @AppStorage(
        AppSettingKey.currentEnv.rawValue,
        store: UserDefaults(suiteName: Constants.groupID.rawValue)
    )
    public var currentEnv: String = "mainnet"

    @AppStorage(AppSettingKey.isZknymEnabled.rawValue)
    public var isZknymEnabled: Bool?

    // Observed values for view models
    @Published public var isErrorReportingOnPublisher = false
    @Published public var isCredentialImportedPublisher = false
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

public enum AppSettingKey: String {
    case currentAppearance
    case appMode
    case errorReporting
    case credenitalExists
    case smallScreen
    case welcomeScreenDidDisplay
    case entryGateway
    case exitRouter
    // TODO: remove after migration. Introduced in 1.6.0
    case entryCountry
    // TODO: remove after migration. Introduced in 1.6.0
    case exitCountry
    case connectionType
    case lastConnectionIntent
    case currentEnv
    case countryStore
    case gatewayStore
    case santaEntryGateways
    case santaExitGateways
    case isZknymEnabled
}
