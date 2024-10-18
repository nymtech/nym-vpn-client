import SwiftUI
import Logging
import AppSettings
import ConfigurationManager
import Home
import Extensions
import KeyboardManager
import Migrations
import NymLogger
import NotificationsManager
import SentryManager
import Theme

@main
struct NymVPNApp: App {
    private let logFileManager = LogFileManager(logFileType: .app)

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
            .environmentObject(logFileManager)
        }
    }
}

private extension NymVPNApp {
    func setup() {
        LoggingSystem.bootstrap { label in
            FileLogHandler(label: label, logFileManager: logFileManager)
        }
        try? ConfigurationManager.shared.setup()
        NotificationsManager.shared.setup()
        ThemeConfiguration.setup()
        SentryManager.shared.setup()
        Migrations.shared.setup()
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
