import SwiftUI
import AppSettings
import Theme
import UIComponents

public struct LaunchView: View {
    @Binding private var splashScreenDidDisplay: Bool

    public init(splashScreenDidDisplay: Binding<Bool>) {
        _splashScreenDidDisplay = splashScreenDidDisplay
    }

    public var body: some View {
        VStack {
            Spacer()
            HStack {
                Spacer()
                SplashAnimationView(
                    viewModel:
                        SplashAnimationViewModel(
                            splashScreenDidDisplay: $splashScreenDidDisplay
                        )
                )
                Spacer()
            }
            Spacer()
        }
        .preferredColorScheme(AppSettings.shared.currentAppearance.colorScheme)
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
    }
}
