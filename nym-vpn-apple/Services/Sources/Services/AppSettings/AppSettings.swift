import SwiftUI

public final class AppSettings: ObservableObject {
    public static let shared = AppSettings()

    #if os(iOS)
    @AppStorage(AppSettingKey.currentAppearance.rawValue)
    public var currentAppearance: AppSetting.Appearance = .automatic
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
    @AppStorage(AppSettingKey.smallScreen.rawValue)
    public var isSmallScreen = false
    @AppStorage(AppSettingKey.welcomeScreenDidDisplay.rawValue)
    public var welcomeScreenDidDisplay = false

    // Observed values for view models
    @Published public var isEntryLocationSelectionOnPublisher = false
    @Published public var isErrorReportingOnPublisher = false

    // Computed properties
    public var isMacOS: Bool {
#if os(macOS)
        return true
#else
        return false
#endif
    }
}

enum AppSettingKey: String {
    case currentAppearance
    case entryLocation
    case errorReporting
    case credenitalExists
    case smallScreen
    case welcomeScreenDidDisplay
}
