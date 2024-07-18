import SwiftUI
import AppSettings
import AutoUpdater
import AutoUpdates
import Constants
import Home
import HelperManager
import SentryManager
import Theme

@main
struct NymVPNDaemonApp: App {
    private let appSettings = AppSettings.shared
    private let autoUpdater = AutoUpdater.shared

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
            .frame(width: 390, height: 800)
            .animation(.default, value: appSettings.welcomeScreenDidDisplay)
            .environmentObject(appSettings)
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
        ThemeConfiguration.setup()
        SentryManager.shared.setup()
        HelperManager.shared.setup(helperName: Constants.helperName.rawValue)
    }
}
