import SwiftUI
import Logging
import AppSettings
import AutoUpdater
import AutoUpdates
import ConfigurationManager
import Constants
import Home
import HelperManager
import NymLogger
import Migrations
import SentryManager
import Theme

@main
struct NymVPNDaemonApp: App {
    private let autoUpdater = AutoUpdater.shared
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
            .frame(width: 390, height: 800)
            .animation(.default, value: appSettings.welcomeScreenDidDisplay)
            .environmentObject(appSettings)
            .environmentObject(logFileManager)
        }
        .windowResizability(.contentSize)
        .commands {
            CommandGroup(after: .appInfo) {
                CheckForUpdatesView(viewModel: CheckForUpdatesViewModel(updater: autoUpdater.updater))
            }
        }
    }
}

private extension NymVPNDaemonApp {
    func setup() {
        LoggingSystem.bootstrap { label in
            FileLogHandler(label: label, logFileManager: logFileManager)
        }
        try? ConfigurationManager.shared.setup()
        ThemeConfiguration.setup()
        SentryManager.shared.setup()
        HelperManager.shared.setup(helperName: Constants.helperName.rawValue)
        Migrations.shared.setup()
    }
}
