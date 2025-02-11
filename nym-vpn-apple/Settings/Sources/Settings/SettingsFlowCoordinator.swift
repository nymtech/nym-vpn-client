import SwiftUI
import AppSettings
#if os(macOS)
import HelperInstall
#endif
import NymLogger

struct SettingsFlowCoordinator<Content: View>: View {
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
        case .theme:
            AppearanceView(viewModel: AppearanceViewModel(path: $flowState.path, appSettings: AppSettings.shared))
        case .support:
            SupportView(viewModel: SupportViewModel(path: $flowState.path))
        case .legal:
            LegalView(viewModel: LegalViewModel(path: $flowState.path))
        case .addCredentials:
            AddCredentialsView(viewModel: AddCredentialsViewModel(path: $flowState.path))
        case .logs:
            LogsView(viewModel: LogsViewModel(path: $flowState.path, logFileManager: logFileManager))
        case .acknowledgments:
            AcknowledgmentsView(viewModel: AcknowledgeMentsViewModel(navigationPath: $flowState.path))
        case let .licence(details: details):
            LicenseView(viewModel: LicenseViewModel(path: $flowState.path, details: details))
        case .santasMenu:
            SantasView(viewModel: SantasViewModel(path: $flowState.path))
#if os(macOS)
        case let .installHelper(afterInstallAction):
            HelperInstallView(
                viewModel: HelperInstallViewModel(
                    path: $flowState.path,
                    afterInstallAction: afterInstallAction
                )
            )
#endif
        }
    }
}
