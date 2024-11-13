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
    private let helperManager = HelperManager.shared
    private let appSettings = AppSettings.shared

    @StateObject private var homeViewModel = HomeViewModel()
    @StateObject private var checkForUpdatesViewModel = CheckForUpdatesViewModel(updater: AutoUpdater.shared.updater)
    @StateObject private var welcomeViewModel = WelcomeViewModel()
    @State private var isDisplayingAlert = false
    @State private var alertTitle = ""

    init() {
        setup()
    }

    var body: some Scene {
        WindowGroup {
            NavigationStack {
                // TODO: create flow coordinator, add to flow coordinator.
                if !homeViewModel.splashScreenDidDisplay {
                    LaunchView(splashScreenDidDisplay: $homeViewModel.splashScreenDidDisplay)
                } else if !appSettings.welcomeScreenDidDisplay {
                    WelcomeView(viewModel: welcomeViewModel)
                        .transition(.slide)
                } else {
                    HomeView(viewModel: homeViewModel)
                        .transition(.slide)
                }
            }
            .alert(alertTitle, isPresented: $isDisplayingAlert) {
                Button("ok".localizedString, role: .cancel) { }
            }
            .frame(width: 390, height: 800)
            .animation(.default, value: appSettings.welcomeScreenDidDisplay)
            .environmentObject(appSettings)
            .environmentObject(logFileManager)
        }
        .windowResizability(.contentSize)
        .commands {
            CommandGroup(after: .appInfo) {
                CheckForUpdatesView(viewModel: checkForUpdatesViewModel)
            }
            CommandGroup(after: .help) {
                if helperManager.isHelperAuthorizedAndRunning() {
                    Button("helper.uninstallHelper".localizedString) {
                        let result = HelperManager.shared.uninstallHelper()
                        if result {
                            alertTitle = "helper.successfullyUninstalled".localizedString
                        } else {
                            alertTitle = "error.unexpected".localizedString
                        }
                        isDisplayingAlert = true
                    }
                }
            }
        }
    }
}

private extension NymVPNDaemonApp {
    func setup() {
        LoggingSystem.bootstrap { label in
            FileLogHandler(label: label, logFileManager: logFileManager)
        }
        Task {
            try await ConfigurationManager.shared.setup()
        }
        ThemeConfiguration.setup()
        SentryManager.shared.setup()
        HelperManager.shared.setup(helperName: Constants.helperName.rawValue)
        Migrations.shared.setup()
    }
}
