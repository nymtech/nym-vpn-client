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
        case .support:
            supportDestination()
        case .legal:
            legalDestination()
        case .addCredentials:
            addCredentialsDestination()
        case .createAccountWelcome:
            createAccountWelcomeDestination()
        case .generatePassphrase:
            GeneratePassphraseView(path: $flowState.path)
        case let .planPurchase(shouldDisplayBackButton: shouldDisplayBackButton):
            PurchasePlanView(path: $flowState.path, shouldDisplayBackButton: shouldDisplayBackButton)
        case .processingAccount:
            ProcessingAccountView(path: $flowState.path)
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
        case .santasMenu:
            santasMenuDestination()
#if os(macOS)
        case .proxy:
            proxyDestination()
        case .appMode:
            appModeDestination()
        case .daemonEnable:
            DaemonInstallView(isServing: $grpcManager.isServing, path: $flowState.path)
#endif
        case .privacyAndData:
            privacyAndDataDestination()
        case .censorship:
            censorshipDestination()
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
    func addCredentialsDestination() -> some View {
#if os(iOS)
        AddCredentialsView(
            viewModel:
                AddCredentialsViewModel(
                    path: $flowState.path,
                    appSettings: .shared,
                    credentialsManager: .shared,
                    configurationManager: .shared,
                    keyboardManager: .shared
                )
        )
#elseif os(macOS)
        AddCredentialsView(
            viewModel: AddCredentialsViewModel(
                path: $flowState.path,
                appSettings: .shared,
                configurationManager: .shared,
                credentialsManager: .shared
            )
        )
#endif
    }

    @ViewBuilder
    func createAccountWelcomeDestination() -> some View {
        CreateAccountWelcomeView(path: $flowState.path)
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

#if os(macOS)
    @ViewBuilder
    func proxyDestination() -> some View {
        ProxyView(path: $flowState.path)
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

    @ViewBuilder
    func censorshipDestination() -> some View {
        CensorshipView(path: $flowState.path)
    }

    func accountAndDevicesDestination() -> some View {
        AccountAndDevicesView(path: $flowState.path)
    }
}
