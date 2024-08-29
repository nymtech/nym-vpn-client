import SwiftUI
import AppSettings
import Theme

public final class AppearanceViewModel: ObservableObject {
    @ObservedObject private var appSettings: AppSettings

    let title = "settings.appearance".localizedString

    @Published var currentAppearance: AppSetting.Appearance
    @Binding var path: NavigationPath

    var themes: [AppSetting.Appearance] {
        AppSetting.Appearance.allCases
    }

    public init(path: Binding<NavigationPath>, appSettings: AppSettings = AppSettings.shared) {
        _path = path
        self.appSettings = appSettings
        currentAppearance = appSettings.currentAppearance
    }

    func updateAppearance(with appearance: AppSetting.Appearance) {
        appSettings.currentAppearance = appearance
        currentAppearance = appearance
    }
}

extension AppearanceViewModel {
    func appearanceTitle(for theme: AppSetting.Appearance) -> String {
        switch theme {
        case .light:
            return "lightThemeTitle".localizedString
        case .dark:
            return "darkThemeTitle".localizedString
        #if os(iOS)
        case .automatic:
            return "automaticThemeTitle".localizedString
        #endif
        }
    }

    func appearanceSubtitle(for theme: AppSetting.Appearance) -> String? {
        switch theme {
        case .light, .dark:
            return nil
        #if os(iOS)
        case .automatic:
            return "automaticThemeSubtitle".localizedString
        #endif
        }
    }
}

// MARK: - Navigation -
extension AppearanceViewModel {
    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }
}
