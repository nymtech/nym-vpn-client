#if os(iOS)
import UIKit
#endif

extension AppearanceView {
    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }

    func navigateToDisplayTheme() {
        path.append(SettingLink.displayTheme)
    }

#if os(iOS)
    func navigateToAppIcon() {
        path.append(SettingLink.appIcon)
    }
#endif

    func navigateToLanguage() {
#if os(iOS)
        try? externalLinkManager.openExternalURL(urlString: UIApplication.openSettingsURLString)
#elseif os(macOS)
        try? externalLinkManager.openExternalURL(urlString: "x-apple.systempreferences:com.apple.Localization")
#endif
    }
}
