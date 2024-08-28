import SwiftUI
import AppSettings

public struct SplashAnimationViewModel {
    let appSettings: AppSettings

    @Binding var splashScreenDidDisplay: Bool

    public init(splashScreenDidDisplay: Binding<Bool>, appSettings: AppSettings = AppSettings.shared) {
        self.appSettings = appSettings
        _splashScreenDidDisplay = splashScreenDidDisplay
    }

    var animationName: String {
        switch appSettings.currentAppearance {
#if os(iOS)
        case .automatic:
            let userInterfaceStyle = UITraitCollection.current.userInterfaceStyle
            return userInterfaceStyle == .dark ? "launchDark" : "launchLight"
#endif
        case .light:
            return "launchLight"
        case .dark:
            return "launchDark"
        }
    }

    @MainActor func didFinishDisplayingAnimation() {
        splashScreenDidDisplay = true
    }
}
