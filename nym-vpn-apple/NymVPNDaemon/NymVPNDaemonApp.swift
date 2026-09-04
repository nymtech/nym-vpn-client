import SwiftUI
import Logging
import AppDiscoveryService
import AppSettings
import AutoUpdater
import AutoUpdates
import ConnectionManager
import ConfigurationManager
import CredentialsManager
import Constants
import DeeplinkManager
import ExternalLinkManager
import FeatureFlagsManager
import GatewayManager
import GRPCManager
import Home
import ImpactGenerator
import NotificationsManager
import NymLogger
import Migrations
import SentryManager
import Theme
import TunnelStatus
import UIComponents

@main
struct NymVPNDaemonApp: App {
    // Must be first, to bootstrap logging
    private let nymLogger = NymLogger()
    private let autoUpdater = AutoUpdater.shared
    private let windowId = "NymVPN"

    @Environment(\.openWindow)
    private var openWindow

    @AppStorage(AppSettingKey.currentAppearance.rawValue)
    private var appearance: AppSetting.Appearance = .automatic

    @NSApplicationDelegateAdaptor(AppDelegate.self)
    private var appDelegate

    @State private var appDiscoveryService = AppDiscoveryService()
    @ObservedObject private var appSettings = AppSettings.shared
    @ObservedObject private var configurationManager = ConfigurationManager.shared
    @ObservedObject private var connectionManager = ConnectionManager.shared
    @ObservedObject private var credentialsManager = CredentialsManager.shared
    @ObservedObject private var externalLinkManager = ExternalLinkManager.shared
    @ObservedObject private var featureFlagsManager = FeatureFlagsManager.shared
    @ObservedObject private var gatewayManager = GatewayManager.shared
    @ObservedObject private var grpcManager = GRPCManager.shared
    @ObservedObject private var impactGenerator = ImpactGenerator.shared
    @State private var deeplinkManager = DeeplinkManager(credentialsManager: CredentialsManager.shared)

    @State private var appFeatureViewModel = AppFeatureViewModel(
        appSettings: .shared,
        credentialsManager: .shared,
        connectionManager: .shared,
        gatewayManager: .shared,
        snackbarManager: .shared,
        impactGenerator: .shared,
        networkMonitor: .shared,
        grpcManager: .shared
    )
    @StateObject private var checkForUpdatesViewModel = CheckForUpdatesViewModel(updater: AutoUpdater.shared.updater)
    @State private var isDisplayingAlert = false
    @State private var alertTitle = ""
    @State private var splashScreenDidDisplay = false
    @State private var menuBarImageName = "menubarDisconnected"
    @State private var menuBarConnectButtonState = ConnectButtonState.connect
    @State private var isMenuBarVisible: Bool
    @State private var isQuitModalDisplayed = false

    init() {
        isMenuBarVisible = AppSettings.shared.appMode == .menubarOnly || AppSettings.shared.appMode == .both
        setup()
    }

    var body: some Scene {
        Window(windowId, id: windowId) {
            ZStack {
                AppFeatureView(viewModel: appFeatureViewModel)
                    .transition(.slide)
                if !splashScreenDidDisplay {
                    LaunchView(splashScreenDidDisplay: $splashScreenDidDisplay)
                        .transition(.opacity)
                }
            }
            .animation(.easeInOut, value: splashScreenDidDisplay)
            .frame(minWidth: MagicNumbers.macMinWidth.rawValue, minHeight: MagicNumbers.macMinHeight.rawValue)
            .onAppear {
                DispatchQueue.main.async {
                    appDelegate.bringWindowToFront()
                }
                externalLinkManager.deeplinkHandler = { url in
                    await deeplinkManager.handleURL(url)
                }
            }
            .onDisappear {
                if autoUpdater.didPrepareForQuit {
                    quitApp()
                }
            }
            .onOpenURL { incomingURL in
                deeplinkManager.handle(url: incomingURL)
            }
            .alert(alertTitle, isPresented: $isDisplayingAlert) {
                Button("ok".localizedString, role: .cancel) { }
            }
            .overlay {
                quitModalOverlay()
            }
            .environmentObject(appSettings)
            .environmentObject(configurationManager)
            .environmentObject(connectionManager)
            .environmentObject(credentialsManager)
            .environmentObject(externalLinkManager)
            .environmentObject(featureFlagsManager)
            .environmentObject(gatewayManager)
            .environmentObject(grpcManager)
            .environmentObject(impactGenerator)
            .environmentObject(nymLogger.logFileManager)
            .environment(deeplinkManager)
            .environment(appDiscoveryService)
        }
        .onChange(of: appSettings.appMode) { _, newMode in
            appDelegate.configureActivationPolicy(with: newMode)
            configureApp(for: AppSettings.shared.appMode)
        }
        .windowResizability(.contentMinSize)
        .defaultSize(width: MagicNumbers.macMinWidth.rawValue, height: MagicNumbers.macMinHeight.rawValue)
        .commands {
            CommandGroup(replacing: .newItem, addition: {})
            CommandGroup(replacing: .appTermination) {
                Button("quit.NymVPN".localizedString) {
                    isQuitModalDisplayed = true
                }
                .keyboardShortcut("q", modifiers: .command)
            }
            CommandGroup(after: .appInfo) {
                CheckForUpdatesView(viewModel: checkForUpdatesViewModel)
            }
        }
        menuBarExtraView()
    }
}

private extension NymVPNDaemonApp {
    func setup() {
        ThemeConfiguration.setup()
        Task {
            // Things dependant on environment beeing set.
            try await ConfigurationManager.shared.setup()
            CredentialsManager.shared.setup()
            FeatureFlagsManager.shared.setup()
            GatewayManager.shared.setup()
            NotificationsManager.shared.setup()
            SentryManager.shared.setup()
            Migrations.shared.setup()
        }
    }
}

private extension NymVPNDaemonApp {
    @ViewBuilder
    func quitModalOverlay() -> some View {
        if isQuitModalDisplayed {
            QuitAppModal(
                isDisplayed: $isQuitModalDisplayed,
                closeAction: {
                    closeWindow()
                }, quitAction: {
                    quitApp()
                }
            )
            .transition(.opacity)
            .animation(.easeInOut, value: isQuitModalDisplayed)
        }
    }
}

// MARK: - Menubar -
private extension NymVPNDaemonApp {
    func configureApp(for mode: AppSetting.AppMode) {
        switch mode {
        case .menubarOnly:
            isMenuBarVisible = true
        case .dockOnly:
            isMenuBarVisible = false
        case .both:
            isMenuBarVisible = true
        }
    }

    func updateImageName(with status: TunnelStatus) {
        switch status {
        case .connected:
            menuBarImageName = "menubarConnected"
        case .connecting, .reasserting, .restarting:
            menuBarImageName = "menubarConnecting"
        case .disconnected, .offline, .offlineReconnect, .unknown:
            menuBarImageName = "menubarDisconnected"
        case .disconnecting:
            menuBarImageName = "menubarDisconnected"
        case .error:
            menuBarImageName = "menubarError"
        }
    }

    var menuBarNSImage: NSImage {
        let image = NSImage(named: menuBarImageName) ?? NSImage()
        image.size = NSSize(width: 25, height: 18)
        image.isTemplate = true
        return image
    }

    func menuBarExtraView() -> some Scene {
        MenuBarExtra(isInserted: $isMenuBarVisible) {
            menuBarItemContent()
        } label: {
            Image(nsImage: menuBarNSImage)
        }
        .menuBarExtraStyle(.menu)
        .onChange(of: connectionManager.currentTunnelStatus, initial: true) { _, status in
            updateImageName(with: status)
            refreshMenuBarConnectButtonState()
        }
        .onChange(of: appSettings.isCredentialImportedPublisher) { _, _ in
            refreshMenuBarConnectButtonState()
        }
        .onChange(of: credentialsManager.accountSummaryLastFetchFailed) { _, _ in
            refreshMenuBarConnectButtonState()
        }
        .onChange(of: credentialsManager.accountSummary?.isActive) { _, _ in
            refreshMenuBarConnectButtonState()
        }
    }

    func refreshMenuBarConnectButtonState() {
        menuBarConnectButtonState = ConnectButtonState(
            tunnelStatus: connectionManager.currentTunnelStatus,
            isCredentialImported: credentialsManager.isValidCredentialImported,
            accountSummaryLastFetchFailed: credentialsManager.accountSummaryLastFetchFailed,
            isAccountActive: credentialsManager.isAccountActive(),
            hasAccountSummary: credentialsManager.accountSummary != nil
        )
    }

    func closeWindow() {
        @Environment(\.dismissWindow)
        var dismissWindow
        dismissWindow(id: windowId)
    }

    func quitApp() {
        appDelegate.shouldTerminate = true
        NSApplication.shared.terminate(self)
    }

    @ViewBuilder
    func menuBarItemContent() -> some View {
        connectDisconnectButton()
        connectionDetails()
        Button("menuBar.openApp".localizedString) {
            appDelegate.bringWindowToFront()
        }
        .keyboardShortcut("o")
        Divider()
        Button("quit.NymVPN".localizedString) {
            quitApp()
        }
    }

    @ViewBuilder
    func connectDisconnectButton() -> some View {
        if menuBarConnectButtonState.menuBarItemIsAction {
            Button(menuBarConnectButtonState.localizedTitle) {
                Task { @MainActor in
                    try? await connectionManager.connectDisconnect()
                }
            }
        } else {
            Text(menuBarConnectButtonState.localizedTitle)
        }
        Divider()
    }

    @ViewBuilder
    func connectionDetails() -> some View {
        if let entryName = gatewayManager.userFriendlyTitle(with: connectionManager.entryGateway),
           let exitName = gatewayManager.userFriendlyTitle(with: connectionManager.exitRouter) {
            let entry = gatewayManager.localizedCountry(with: entryName)?.name ?? entryName
            let exit = gatewayManager.localizedCountry(with: exitName)?.name ?? exitName

            let statusButtonConfig = StatusButtonConfig(
                tunnelStatus: connectionManager.currentTunnelStatus,
                hasInternet: true
            )

            if connectionManager.currentTunnelStatus == .connected {
                Text("\(statusButtonConfig.rawValue.localizedString)")
                Text("\("home.entryHop".localizedString): \(entry)")
                Text("\("home.exitHop".localizedString): \(exit)")
                Divider()
            }
        }
    }
}
