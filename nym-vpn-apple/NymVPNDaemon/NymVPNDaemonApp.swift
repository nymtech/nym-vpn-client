import SwiftUI
import Logging
import AppSettings
import AutoUpdater
import AutoUpdates
import ConnectionManager
import ConfigurationManager
import Constants
import FeatureFlagsManager
import GatewayManager
import GRPCManager
import Home
import HelperManager
import NotificationsManager
import NymLogger
import MessagesManager
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
    private var appearance: AppSetting.Appearance = .light

    @NSApplicationDelegateAdaptor(AppDelegate.self)
    private var appDelegate

    @ObservedObject private var appSettings = AppSettings.shared
    @ObservedObject private var connectionManager = ConnectionManager.shared
    @ObservedObject private var grpcManager = GRPCManager.shared
    @ObservedObject private var featureFlagsManager = FeatureFlagsManager.shared
    @ObservedObject private var gatewayManager = GatewayManager.shared
    @StateObject private var homeViewModel = HomeViewModel()
    @StateObject private var checkForUpdatesViewModel = CheckForUpdatesViewModel(updater: AutoUpdater.shared.updater)
    @StateObject private var welcomeViewModel = WelcomeViewModel()
    @State private var isDisplayingAlert = false
    @State private var alertTitle = ""
    @State private var splashScreenDidDisplay = false
    @State private var menuBarImageName = "NymLogoDisabled"
    @State private var menuBarConnectButtonState = ConnectButtonState.connect
    @State private var isMenuBarVisible: Bool
    @State private var isQuitModalDisplayed = false

    init() {
        isMenuBarVisible = AppSettings.shared.appMode == .menubarOnly || AppSettings.shared.appMode == .both
        setup()
    }

    var body: some Scene {
        Window(windowId, id: windowId) {
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
            .frame(minWidth: MagicNumbers.macMinWidth.rawValue, minHeight: MagicNumbers.macMinHeight.rawValue)
            .onAppear {
                DispatchQueue.main.async {
                    appDelegate.bringWindowToFront()
                }
            }
            .onDisappear {
                if autoUpdater.didPrepareForQuit {
                    quitApp()
                }
            }
            .alert(alertTitle, isPresented: $isDisplayingAlert) {
                Button("ok".localizedString, role: .cancel) { }
            }
            .overlay {
                quitModalOverlay()
            }
            .preferredColorScheme(appearance.colorScheme)
            .animation(.default, value: appSettings.welcomeScreenDidDisplay)
            .environmentObject(appSettings)
            .environmentObject(connectionManager)
            .environmentObject(featureFlagsManager)
            .environmentObject(gatewayManager)
            .environmentObject(grpcManager)
            .environmentObject(nymLogger.logFileManager)
        }
        .onChange(of: appSettings.appMode) { newMode in
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
        menuBarExtraView()
    }
}

private extension NymVPNDaemonApp {
    func setup() {
        ThemeConfiguration.setup()
        Task {
            // Things dependant on environment beeing set.
            try await ConfigurationManager.shared.setup(for: .main)
            FeatureFlagsManager.shared.setup()
            GatewayManager.shared.setup()
            MessagesManager.shared.setup()
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
        menuBarImageName = status == .connected ? "NymLogo" : "NymLogoDisabled"
    }

    func menuBarExtraView() -> some Scene {
        MenuBarExtra(isInserted: $isMenuBarVisible) {
            menuBarItemContent()
        } label: {
            Image(menuBarImageName)
                .renderingMode(.template)
                .frame(width: 32)
                .foregroundStyle(.primary)
        }
        .menuBarExtraStyle(.menu)
        .onChange(of: connectionManager.currentTunnelStatus) { status in
            updateImageName(with: status)
            menuBarConnectButtonState = ConnectButtonState(tunnelStatus: status)
        }
    }

    func closeWindow() {
        if #available(macOS 14.0, *) {
            @Environment(\.dismissWindow)
            var dismissWindow
            dismissWindow(id: windowId)
        } else {
            NSApplication.shared.windows
                .first(where: { $0.identifier?.rawValue == windowId })?
                .close()
        }
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
        let entryName = connectionManager.entryGateway.name
        let entry = gatewayManager.country(with: entryName)?.name ?? entryName

        let exitName = connectionManager.exitRouter.name
        let exit = gatewayManager.country(with: exitName)?.name ?? exitName

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
