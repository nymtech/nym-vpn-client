import SwiftUI
import AppSettings
import Theme
import UIComponents

public struct LaunchView: View {
    @Binding private var splashScreenDidDisplay: Bool
    @State private var logoOpacity: Double = 0.0

    public init(splashScreenDidDisplay: Binding<Bool>) {
        _splashScreenDidDisplay = splashScreenDidDisplay
    }

    public var body: some View {
        LogoView()
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
                    }
                }
            }
    }
}
