import SwiftUI
import Logging
import AppSettings
import ConfigurationManager
import ConnectionManager
import CredentialsManager
import DeeplinkManager
import ExternalLinkManager
import FeatureFlagsManager
import GatewayManager
import Home
import ImpactGenerator
import Extensions
import KeyboardManager
import MessagesManager
import Migrations
import NymLogger
import NotificationsManager
import PurchasesManager
import SentryManager
import Theme
#if os(iOS)
import NymVPNLib
#endif

#if os(iOS)
var runtimeInit: () = {
    initializeTokioRuntime()
    initLogger(logDir: nil, logLevel: .debug, sentryMonitoring: false)
}()
#endif

@main
struct NymVPNApp: App {
#if os(iOS)
    private let runtimeInitOnce: () = runtimeInit
#endif
    private let logFileManager = LogFileManager(logFileType: .app)

    @AppStorage(AppSettingKey.currentAppearance.rawValue)
    private var appearance: AppSetting.Appearance = .automatic

    @Environment(\.scenePhase)
    private var scenePhase

    @ObservedObject private var appSettings = AppSettings.shared
    @ObservedObject private var configurationManager = ConfigurationManager.shared
    @ObservedObject private var connectionManager = ConnectionManager.shared
    @ObservedObject private var credentialsManager = CredentialsManager.shared
    @ObservedObject private var featureFlagsManager = FeatureFlagsManager.shared
    @ObservedObject private var externalLinkManager = ExternalLinkManager.shared
    @ObservedObject private var gatewayManager = GatewayManager.shared
    @ObservedObject private var impactGenerator = ImpactGenerator.shared
    @ObservedObject private var purchasesManager = PurchasesManager()
    @State private var deeplinkManager = DeeplinkManager(credentialsManager: CredentialsManager.shared)

    @ObservedObject private var homeViewModel = HomeViewModel(
        appSettings: .shared,
        connectionManager: .shared,
        configurationManager: .shared,
        credentialsManager: .shared,
        networkMonitor: .shared,
        externalLinkManager: .shared,
        gatewayManager: .shared,
        impactGenerator: .shared,
        messagesManager: .shared
    )

    @State private var isSecureScreenVisible = false

    init() {
        setup()
    }

    var body: some Scene {
        WindowGroup {
            NavigationStack {
                HomeView(viewModel: homeViewModel)
                    .transition(.slide)
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
            .onOpenURL { incomingURL in
                deeplinkManager.handle(url: incomingURL)
            }
            .environmentObject(appSettings)
            .environmentObject(configurationManager)
            .environmentObject(connectionManager)
            .environmentObject(credentialsManager)
            .environmentObject(externalLinkManager)
            .environmentObject(featureFlagsManager)
            .environmentObject(gatewayManager)
            .environmentObject(impactGenerator)
            .environmentObject(KeyboardManager.shared)
            .environmentObject(logFileManager)
            .environmentObject(purchasesManager)
            .environment(deeplinkManager)
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
            FeatureFlagsManager.shared.setup()
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
