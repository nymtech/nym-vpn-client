import SwiftUI

public final class AppSettings: ObservableObject {
    public static let shared = AppSettings()

    @AppStorage("currentAppearance") public var currentAppearance: AppSetting.Appearance = .automatic
}
