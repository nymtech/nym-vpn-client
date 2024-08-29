import SwiftUI
import Logging
import AppSettings
import ConfigurationManager
import Home
import Extensions
import KeyboardManager
import NymLogger
import SentryManager
import Theme

@main
struct NymVPNApp: App {
    @ObservedObject private var appSettings = AppSettings.shared
    @StateObject private var homeViewModel = HomeViewModel()

    init() {
        setup()
    }

    var body: some Scene {
        WindowGroup {
            NavigationStack {
                if !homeViewModel.splashScreenDidDisplay {
                    LaunchView(splashScreenDidDisplay: $homeViewModel.splashScreenDidDisplay)
                } else if !appSettings.welcomeScreenDidDisplay {
                    WelcomeView(viewModel: WelcomeViewModel())
                        .transition(.slide)
                } else {
                    HomeView(viewModel: homeViewModel)
                        .transition(.slide)
                }
            }
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
        LoggingSystem.bootstrap { label in
            FileLogHandler(label: label)
        }
        try? ConfigurationManager.setEnvVariables(for: .mainnet)
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
