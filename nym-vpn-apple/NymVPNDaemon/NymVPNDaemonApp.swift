import SwiftUI
import Logging
import AppSettings
import AutoUpdater
import AutoUpdates
import ConfigurationManager
import Constants
import Home
import HelperManager
import NotificationsManager
import NymLogger
import Migrations
import SentryManager
import SystemMessageManager
import Theme

@main
struct NymVPNDaemonApp: App {
    private let autoUpdater = AutoUpdater.shared
    private let logFileManager = LogFileManager(logFileType: .app)

    @AppStorage(AppSettingKey.currentAppearance.rawValue)
    private var appearance: AppSetting.Appearance = .light

    @ObservedObject private var appSettings = AppSettings.shared
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
                if !appSettings.welcomeScreenDidDisplay {
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
            .preferredColorScheme(appearance.colorScheme)
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

private extension NymVPNDaemonApp {
    func setup() {
        LoggingSystem.bootstrap { label in
            FileLogHandler(label: label, logFileManager: logFileManager)
        }
        Task {
            // Things dependant on environment beeing set.
            try await ConfigurationManager.shared.setup()
            SystemMessageManager.shared.setup()
        }
        NotificationsManager.shared.setup()
        ThemeConfiguration.setup()
        SentryManager.shared.setup()
        HelperManager.shared.setup(helperName: Constants.helperName.rawValue)
        Migrations.shared.setup()
    }
}
