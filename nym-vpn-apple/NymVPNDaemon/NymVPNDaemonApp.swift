import SwiftUI
import Logging
import AppSettings
import AutoUpdater
import AutoUpdates
import ConnectionManager
import ConfigurationManager
import Constants
import CountriesManager
import GatewayManager
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
    @ObservedObject private var connectionManager = ConnectionManager.shared
    @ObservedObject private var countriesManager = CountriesManager.shared
    @StateObject private var homeViewModel = HomeViewModel()
    @StateObject private var checkForUpdatesViewModel = CheckForUpdatesViewModel(updater: AutoUpdater.shared.updater)
    @StateObject private var welcomeViewModel = WelcomeViewModel()
    @State private var isDisplayingAlert = false
    @State private var alertTitle = ""
    @State private var splashScreenDidDisplay = false

    init() {
        setup()
    }

    var body: some Scene {
        WindowGroup {
            NavigationStack {
                if !splashScreenDidDisplay {
                    LaunchView(splashScreenDidDisplay: $splashScreenDidDisplay)
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
            .preferredColorScheme(appearance.colorScheme)
            .frame(width: 390, height: 800)
            .animation(.default, value: appSettings.welcomeScreenDidDisplay)
            .environmentObject(appSettings)
            .environmentObject(connectionManager)
            .environmentObject(countriesManager)
            .environmentObject(logFileManager)
        }
        .windowResizability(.contentSize)
        .commands {
            CommandGroup(replacing: .newItem, addition: {})
            CommandGroup(after: .appInfo) {
                CheckForUpdatesView(viewModel: checkForUpdatesViewModel)
            }
            CommandGroup(after: .help) {
                Button("helper.uninstallHelper".localizedString) {
                    Task {
                        do {
                            try await HelperManager.shared.uninstall()
                            alertTitle = "helper.successfullyUninstalled".localizedString
                        } catch {
                            alertTitle = error.localizedDescription
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
        ThemeConfiguration.setup()
        Task {
            // Things dependant on environment beeing set.
            try await ConfigurationManager.shared.setup()
            CountriesManager.shared.setup()
            GatewayManager.shared.setup()
            SystemMessageManager.shared.setup()
            NotificationsManager.shared.setup()
            SentryManager.shared.setup()
            Migrations.shared.setup()
        }
    }
}
