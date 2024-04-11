import SwiftUI

public final class AppSettings: ObservableObject {
    public static let shared = AppSettings()

    #if os(iOS)
    @AppStorage("currentAppearance") public var currentAppearance: AppSetting.Appearance = .automatic
    #else
    @AppStorage("currentAppearance") public var currentAppearance: AppSetting.Appearance = .light
    #endif
    @AppStorage("entryLocation") public var entryLocationSelectionIsOn = false
    @AppStorage("errorReporting") public var errorReportingIsOn = false
}
