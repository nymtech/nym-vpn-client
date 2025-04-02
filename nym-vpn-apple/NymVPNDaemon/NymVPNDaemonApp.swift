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
import GRPCManager
import Home
import HelperManager
import NotificationsManager
import NymLogger
import Migrations
import SentryManager
import SystemMessageManager
import Theme
import TunnelStatus
import UIComponents

@main
struct NymVPNDaemonApp: App {
    private let autoUpdater = AutoUpdater.shared
    private let logFileManager = LogFileManager(logFileType: .app)
    private let windowId = "NymVPN"
    private let grpcManager = GRPCManager.shared

    @Environment(\.openWindow)
    private var openWindow

    @AppStorage(AppSettingKey.currentAppearance.rawValue)
    private var appearance: AppSetting.Appearance = .light

    @NSApplicationDelegateAdaptor(AppDelegate.self)
    private var appDelegate

    @ObservedObject private var appSettings = AppSettings.shared
    @ObservedObject private var connectionManager = ConnectionManager.shared
    @ObservedObject private var countriesManager = CountriesManager.shared
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
        switch AppSettings.shared.appMode {
        case .both, .menubarOnly:
            isMenuBarVisible = true
        case .dockOnly:
            isMenuBarVisible = false
        }
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
                configureApp(for: appSettings.appMode)
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
            .environmentObject(countriesManager)
            .environmentObject(logFileManager)
        }
        .onChange(of: appSettings.appMode) { newMode in
            configureApp(for: newMode)
        }
        .windowResizability(.contentMinSize)
        .defaultSize(width: MagicNumbers.macMinWidth.rawValue, height: MagicNumbers.macMinHeight.rawValue)
        .commands {
            CommandGroup(replacing: .newItem, addition: {})
            CommandGroup(replacing: .appTermination) {
                Button("quit.NymVPN".localizedString) {
                    isQuitModalDisplayed = true
                }
                .keyboardShortcut("q")
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
            NSApp.setActivationPolicy(.accessory)
            isMenuBarVisible = true
        case .dockOnly:
            NSApp.setActivationPolicy(.regular)
            isMenuBarVisible = false
        case .both:
            NSApp.setActivationPolicy(.regular)
            isMenuBarVisible = true
        }
    }

    func bringWindowToFront() {
        NSApp.windows.first?.makeKeyAndOrderFront(self)
        if #available(macOS 14.0, *) {
            NSApplication.shared.activate()
        } else {
            NSApp.activate(ignoringOtherApps: true)
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
            bringWindowToFront()
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
        if connectionManager.currentTunnelStatus == .connected,
           let connectedDateString = connectionManager.connectedDateString {
            Text("\("connectionTime".localizedString): \(connectedDateString)")
            Text("\("home.entryHop".localizedString): \(connectionManager.entryGateway.name)")
            Text("\("home.exitHop".localizedString): \(connectionManager.exitRouter.name)")
            Divider()
        }
    }
}
