import SwiftUI
import Logging
import AccountPrefetchGates
import AppSettings
import ConfigurationManager
import ConnectionManager
import Constants
import CredentialsManager
import DeeplinkManager
import ExternalLinkManager
import FeatureFlagsManager
import GatewayManager
import Home
import ImpactGenerator
import Extensions
import KeyboardManager
import Migrations
import NymLogger
import NotificationsManager
import PurchasesManager
import SentryManager
import Theme
import NymVPNLib

@main
struct NymVPNApp: App {
    private let logFileManager: LogFileManager = {
        let manager = LogFileManager(logFileType: .app)
        initLogger(logDir: LogFileManager.logsDirectory()?.path(), logLevel: .debug, sentryMonitoring: false)
        LoggingSystem.bootstrap { label in
            let fileLogger = FileLogHandler(label: label, logFileManager: manager)

            #if DEBUG
                return MultiplexLogHandler([
                    StreamLogHandler.standardOutput(label: label),
                    fileLogger
                ])
            #else
                return fileLogger
            #endif
        }
        return manager
    }()

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

    @State private var appFeatureViewModel = AppFeatureViewModel(
        appSettings: .shared,
        credentialsManager: .shared,
        connectionManager: .shared,
        gatewayManager: .shared,
        snackbarManager: .shared,
        impactGenerator: .shared,
        networkMonitor: .shared
    )

    @State private var isSecureScreenVisible = false
    @State private var splashScreenDidDisplay = false

    init() {
        setup()
    }

    var body: some Scene {
        WindowGroup {
            ZStack {
                AppFeatureView(viewModel: appFeatureViewModel)
                    .transition(.slide)
                if !splashScreenDidDisplay {
                    LaunchView(splashScreenDidDisplay: $splashScreenDidDisplay)
                        .transition(.opacity)
                }
            }
            .animation(.easeInOut, value: splashScreenDidDisplay)
            .onChange(of: scenePhase) { _, newPhase in
                configureSecureScreen(with: newPhase)
#if os(iOS)
                if newPhase == .background {
                    credentialsManager.shutdownControllers()
                    BackgroundRefreshScheduler.scheduleAppRefresh()
                }
#endif
            }
            .inAppSafari(using: externalLinkManager)
            .overlay {
                if isSecureScreenVisible {
                    LogoView()
                }
            }
            .preferredColorScheme(appearance.colorScheme)
            .onAppear {
                configureScreenSize()
                externalLinkManager.deeplinkHandler = { url in
                    await deeplinkManager.handleURL(url)
                }
                deeplinkManager.onPrivyLoginDeeplink = { callbackURLString in
                    appFeatureViewModel.beginPrivyLoginProcessing(callbackURLString: callbackURLString)
                }
            }
            .onOpenURL { incomingURL in
                if WebCheckoutReturnPolicy.shouldDismissOnDeeplink(url: incomingURL) {
                    externalLinkManager.dismissActiveWebCheckoutSessions()
                } else if incomingURL.scheme == Constants.appUrlScheme.rawValue {
                    externalLinkManager.inAppSafariURL = nil
                }
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
#if os(iOS)
        .backgroundTask(.appRefresh(BackgroundRefreshScheduler.appRefreshIdentifier)) {
            await BackgroundRefreshScheduler.runRefresh()
        }
#endif
    }
}

private extension NymVPNApp {
    func setup() {
        ThemeConfiguration.setup()

        Task {
            // Things dependant on environment being set.
            try await ConfigurationManager.shared.setup()
            CredentialsManager.shared.setup()
            FeatureFlagsManager.shared.setup()
            GatewayManager.shared.setup()
            NotificationsManager.shared.setup()
            SentryManager.shared.setup()
            Migrations.shared.setup()
#if os(iOS) && SANTA
            purchasesManager.registerForEnvironmentChanges(
                configurationManager: configurationManager
            )
#endif
#if os(iOS)
            BackgroundRefreshScheduler.scheduleAppRefresh()
#endif
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
        var transaction = Transaction()
        transaction.disablesAnimations = true
        withTransaction(transaction) {
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
}
