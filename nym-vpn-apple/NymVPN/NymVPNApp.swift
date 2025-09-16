import SwiftUI
import Logging
import AppSettings
import ConfigurationManager
import ConnectionManager
import CountriesManager
import CredentialsManager
import GatewayManager
import Home
import Extensions
import KeyboardManager
import MessagesManager
import Migrations
import NymLogger
import NotificationsManager
import PurchasesManager
import SentryManager
import Theme

@main
struct NymVPNApp: App {
    private let logFileManager = LogFileManager(logFileType: .app)

    @AppStorage(AppSettingKey.currentAppearance.rawValue)
    private var appearance: AppSetting.Appearance = .automatic

    @Environment(\.scenePhase)
    private var scenePhase

    @ObservedObject private var appSettings = AppSettings.shared
    @ObservedObject private var connectionManager = ConnectionManager.shared
    @ObservedObject private var credentialsManager = CredentialsManager.shared
    @ObservedObject private var countriesManager = CountriesManager.shared
    @ObservedObject private var purchasesManager = PurchasesManager()

    @StateObject private var homeViewModel = HomeViewModel()
    @StateObject private var welcomeViewModel = WelcomeViewModel()

    @State private var splashScreenDidDisplay = false
    @State private var isSecureScreenVisible = false

    init() {
        setup()
    }

    var body: some Scene {
        // swiftlint:disable:next closure_body_length
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
            .onChange(of: scenePhase) { _, newPhase in
                configureSecureScreen(with: newPhase)
            }
            .overlay {
                if isSecureScreenVisible {
                    LogoView()
                }
            }
            .animation(.easeIn, value: isSecureScreenVisible)
            .preferredColorScheme(appearance.colorScheme)
            .onAppear {
                configureScreenSize()
            }
            .environmentObject(appSettings)
            .environmentObject(connectionManager)
            .environmentObject(countriesManager)
            .environmentObject(credentialsManager)
            .environmentObject(KeyboardManager.shared)
            .environmentObject(logFileManager)
            .environmentObject(purchasesManager)
        }
    }
}

private extension NymVPNApp {
    func setup() {
        LoggingSystem.bootstrap { label in
            FileLogHandler(label: label, logFileManager: logFileManager)
        }
        ThemeConfiguration.setup()

        Task {
            // Things dependant on environment being set.
            try await ConfigurationManager.shared.setup(for: .main)
            CountriesManager.shared.setup()
            GatewayManager.shared.setup()
            MessagesManager.shared.setup()
            NotificationsManager.shared.setup()
            SentryManager.shared.setup()
            Migrations.shared.setup()
        }
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

    func configureSecureScreen(with newPhase: ScenePhase) {
        switch newPhase {
        case .background, .inactive:
            isSecureScreenVisible = true
        case .active:
            isSecureScreenVisible = false
        @unknown default:
            break
        }
    }
}
