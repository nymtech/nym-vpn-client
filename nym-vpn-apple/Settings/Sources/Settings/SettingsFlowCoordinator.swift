import SwiftUI
import AppSettings
#if os(macOS)
import GRPCManager
#endif
import NymLogger

struct SettingsFlowCoordinator<Content: View>: View {
#if os(macOS)
    @EnvironmentObject private var grpcManager: GRPCManager
#endif
    @EnvironmentObject private var logFileManager: LogFileManager

    @ObservedObject var flowState: SettingsFlowState
    let content: () -> Content

    var body: some View {
        content()
            .navigationDestination(for: SettingLink.self, destination: linkDestination)
    }

    @ViewBuilder
    private func linkDestination(link: SettingLink) -> some View {
        switch link {
        case .appearance:
            appearanceDestination()
        case .displayTheme:
            displayThemeDestination()
#if os(iOS)
        case .appIcon:
            AppIconView(
                viewModel: AppIconViewModel(
                    path: $flowState.path,
                    changer: UIApplicationAppIconChanger()
                )
            )
#endif
        case .support:
            supportDestination()
        case .legal:
            legalDestination()
        case let .addCredentials(navigationSource: navigationSource):
            addCredentialsDestination(navigationSource: navigationSource)
        case let .accountWelcome(type: type, navigationSource: navigationSource):
            accountWelcomeDestination(type: type, navigationSource: navigationSource)
        case let .generatePassphrase(displayPurchaseView: displayPurchaseView):
            GeneratePassphraseView(
                path: $flowState.path,
                displayPurchaseView: displayPurchaseView,
                onPurchaseFlowDismissed: {
                    flowState.onSessionEvent?(.checkoutDismissed)
                }
            )
        case .processingAccount:
            ProcessingAccountView(
                path: $flowState.path,
                onPurchaseFlowComplete: {
                    flowState.onSessionEvent?(.checkoutCompleted)
                },
                onPurchaseFlowDismissed: {
                    flowState.onSessionEvent?(.checkoutDismissed)
                }
            )
        case .passphrase:
            PassphraseView(path: $flowState.path)
        case .logs:
            logsDestination()
        case .acknowledgments:
            acknowledgmentsDestination()
        case let .licence(details: details):
            LicenseView(
                viewModel: LicenseViewModel(
                    path: $flowState.path,
                    details: details,
                    externalLinkManager: .shared
                )
            )
#if SANTA
        case .santasMenu:
            santasMenuDestination()
#endif
#if os(macOS)
        case .proxy:
            proxyDestination()
        case .appMode:
            appModeDestination()
        case .daemonEnable:
            DaemonInstallView(isServing: $grpcManager.isServing, path: $flowState.path)
        case .geoExclusion:
            GeoExclusionView(
                viewModel: GeoExclusionViewModel(
                    path: $flowState.path,
                    connectionManager: .shared,
                    grpcManager: .shared,
                    impactGenerator: .shared
                )
            )
        case let .geoExclusionSetup(port: port):
            GeoExclusionInstructionsView(path: $flowState.path, listenPort: port)
        case .splitTunnel:
            SplitTunnelView(path: $flowState.path)
#endif
        case .diagnosticTool:
            DiagnosticToolView(path: $flowState.path)
        case .privacyAndData:
            privacyAndDataDestination()
        case .dns:
            dnsDestination()
        case .mixnetTuning:
            mixnetTuningDestination()
        case .censorship:
            censorshipDestination()
        case .notifications:
            notificationsDestination()
        case .accountAndDevices:
            accountAndDevicesDestination()
        case .systemStatus:
            SystemStatusView(path: $flowState.path)
        }
    }
}

// MARK: - Private Destinations

private extension SettingsFlowCoordinator {
    @ViewBuilder
    func appearanceDestination() -> some View {
        AppearanceView(path: $flowState.path)
    }

    @ViewBuilder
    func displayThemeDestination() -> some View {
        DisplayThemeView(
            viewModel: DisplayThemeViewModel(
                path: $flowState.path,
                appSettings: AppSettings.shared
            )
        )
    }

    @ViewBuilder
    func supportDestination() -> some View {
        SupportView(
            viewModel: SupportViewModel(
                path: $flowState.path,
                externalLinkManager: .shared
            )
        )
    }

    @ViewBuilder
    func legalDestination() -> some View {
        LegalView(
            viewModel: LegalViewModel(
                path: $flowState.path,
                externalLinkManager: .shared
            )
        )
    }

    @ViewBuilder
    func addCredentialsDestination(navigationSource: AddCredentialsNavigationSource) -> some View {
#if os(iOS)
        AddCredentialsView(
            viewModel:
                AddCredentialsViewModel(
                    path: $flowState.path,
                    appSettings: .shared,
                    credentialsManager: .shared,
                    configurationManager: .shared,
                    keyboardManager: .shared,
                    navigationSource: navigationSource
                )
        )
#elseif os(macOS)
        AddCredentialsView(
            viewModel: AddCredentialsViewModel(
                path: $flowState.path,
                appSettings: .shared,
                configurationManager: .shared,
                credentialsManager: .shared,
                navigationSource: navigationSource
            )
        )
#endif
    }

    @ViewBuilder
    func accountWelcomeDestination(
        type: AccountWelcomeType,
        navigationSource: AccountWelcomeNavigationSource
    ) -> some View {
        AccountWelcomeView(path: $flowState.path, type: type, navigationSource: navigationSource)
    }

    @ViewBuilder
    func logsDestination() -> some View {
#if os(iOS)
        LogsView(
            viewModel: LogsViewModel(
                path: $flowState.path,
                logFileManager: logFileManager,
                impactGenerator: .shared
            )
        )
#elseif os(macOS)
        LogsView(
            viewModel: LogsViewModel(
                path: $flowState.path,
                logFileManager: logFileManager,
                impactGenerator: .shared,
                grpcManager: .shared
            )
        )
#endif
    }

    @ViewBuilder
    func acknowledgmentsDestination() -> some View {
        AcknowledgmentsView(
            viewModel: AcknowledgeMentsViewModel(
                navigationPath: $flowState.path
            )
        )
    }

#if SANTA
    @ViewBuilder
    func santasMenuDestination() -> some View {
#if os(iOS)
        SantasView(
            viewModel: SantasViewModel(
                path: $flowState.path,
                appSettings: .shared,
                configurationManager: .shared
            )
        )
#elseif os(macOS)
        SantasView(
            viewModel: SantasViewModel(
                path: $flowState.path,
                appSettings: .shared,
                configurationManager: .shared,
                grpcManager: .shared
            )
        )
#endif
    }
#endif

#if os(macOS)
    @ViewBuilder
    func proxyDestination() -> some View {
        ProxyView(
            viewModel: ProxyViewModel(
                path: $flowState.path,
                appSettings: .shared,
                connectionManager: .shared,
                grpcManager: .shared
            )
        )
    }

    @ViewBuilder
    func appModeDestination() -> some View {
        AppModeView(path: $flowState.path)
    }
#endif

    @ViewBuilder
    func privacyAndDataDestination() -> some View {
        PrivacyAndDataView(path: $flowState.path)
    }

    func mixnetTuningDestination() -> some View {
        MixnetTuningView(path: $flowState.path)
    }

    @ViewBuilder
    func dnsDestination() -> some View {
        #if os(macOS)
        DnsView(
            viewModel: DnsViewModel(
                path: $flowState.path,
                appSettings: .shared,
                connectionManager: .shared,
                grpcManager: .shared
            )
        )
        #elseif os(iOS)
        DnsView(
            viewModel: DnsViewModel(
                path: $flowState.path,
                appSettings: .shared,
                connectionManager: .shared
            )
        )
        #endif
    }

    @ViewBuilder
    func censorshipDestination() -> some View {
        CensorshipView(path: $flowState.path)
    }

    @ViewBuilder
    func notificationsDestination() -> some View {
        NotificationsView(
            viewModel: NotificationsViewModel(
                path: $flowState.path,
                appSettings: .shared,
                connectionManager: .shared
            )
        )
    }

    func accountAndDevicesDestination() -> some View {
        AccountAndDevicesView(path: $flowState.path)
    }
}
