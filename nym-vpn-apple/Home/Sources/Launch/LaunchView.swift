import SwiftUI
import AppSettings
import Routes
import Theme
import UIComponents

public struct LaunchView: View {
    @EnvironmentObject private var appSettings: AppSettings
    @Binding private var splashScreenDidDisplay: Bool
    @Binding private var path: NavigationPath
    @State private var logoOpacity: Double = 0.0

    public init(splashScreenDidDisplay: Binding<Bool>, path: Binding<NavigationPath>) {
        _splashScreenDidDisplay = splashScreenDidDisplay
        _path = path
    }

    public var body: some View {
        LogoView()
            .navigationBarBackButtonHidden(true)
            .opacity(logoOpacity)
            .background {
                NymColor.background
                    .ignoresSafeArea()
            }
            .task {
                withAnimation(.easeOut(duration: 0.7)) {
                    logoOpacity = 1.0
                } completion: {
                    Task {
                        try? await Task.sleep(for: .seconds(0.3))
                        splashScreenDidDisplay = true
                        path = !appSettings.onboardingDidDisplay ? NavigationPath([HomeLink.onboarding]) : .init()
                    }
                }
            }
    }
}
