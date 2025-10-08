import SwiftUI
import AppSettings

@MainActor public struct SplashAnimationViewModel {
    let appSettings: AppSettings

    @Binding var splashScreenDidDisplay: Bool

    public init(splashScreenDidDisplay: Binding<Bool>, appSettings: AppSettings) {
        self.appSettings = appSettings
        _splashScreenDidDisplay = splashScreenDidDisplay
    }

    let animationName = "launchAnimation"

    @MainActor func didFinishDisplayingAnimation() {
        splashScreenDidDisplay = true
    }
}
