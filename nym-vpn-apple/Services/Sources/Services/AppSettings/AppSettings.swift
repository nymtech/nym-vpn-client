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
    @AppStorage(AppSettingKey.entryLocation.rawValue) public var isEntryLocationSelectionOn = false {
        didSet {
            isEntryLocationSelectionOnPublisher = isEntryLocationSelectionOn
        }
    }
    @AppStorage(AppSettingKey.errorReporting.rawValue) public var isErrorReportingOn = false {
        didSet {
            isErrorReportingOnPublisher = isErrorReportingOn
        }
    }
    @AppStorage(AppSettingKey.credenitalExists.rawValue) public var isCredentialImported = false

    // Observed values for view models
    @Published public var isEntryLocationSelectionOnPublisher = false
    @Published public var isErrorReportingOnPublisher = false
}

enum AppSettingKey: String {
    case currentAppearance
    case entryLocation
    case errorReporting
    case credenitalExists
}
