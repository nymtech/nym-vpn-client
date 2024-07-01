import SwiftUI
import AppSettings
import Home
import Extensions
import KeyboardManager
import SentryManager
import Theme

@main
struct NymVPNApp: App {
    @StateObject private var appSettings = AppSettings.shared
    @StateObject private var homeViewModel = HomeViewModel()

    init() {
        setup()
    }

    var body: some Scene {
        WindowGroup {
            NavigationStack {
                if !appSettings.welcomeScreenDidDisplay {
                    WelcomeView(viewModel: WelcomeViewModel())
                        .transition(.slide)
                } else {
                    HomeView(viewModel: homeViewModel)
                        .transition(.slide)
                }
            }
            .animation(.default, value: appSettings.welcomeScreenDidDisplay)
            .onAppear {
                configureScreenSize()
            }
            .environmentObject(appSettings)
            .environmentObject(KeyboardManager.shared)
        }
    }
}

private extension NymVPNApp {
    func setup() {
        ThemeConfiguration.setup()
        SentryManager.shared.setup()
    }

    func configureScreenSize() {
        guard let screenSize = UIScreen.current?.bounds.size,
              screenSize.width <= 375 && screenSize.height <= 667,
              AppSettings.shared.isSmallScreen != true
        else {
            return
        }
        appSettings.isSmallScreen = true
    }
}
