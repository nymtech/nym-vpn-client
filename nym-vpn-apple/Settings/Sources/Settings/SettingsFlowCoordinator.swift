import SwiftUI
import AppSettings
import NymLogger

struct SettingsFlowCoordinator<Content: View>: View {
    @ObservedObject var flowState: SettingsFlowState
    let content: () -> Content

    var body: some View {
        content()
            .navigationDestination(for: SettingsLink.self, destination: linkDestination)
    }

    @ViewBuilder
    private func linkDestination(link: SettingsLink) -> some View {
        switch link {
        case .theme:
            AppearanceView(viewModel: AppearanceViewModel(path: $flowState.path, appSettings: AppSettings.shared))
        case .feedback:
            FeedbackView(viewModel: FeedbackViewModel(path: $flowState.path))
        case .support:
            SupportView(viewModel: SupportViewModel(path: $flowState.path))
        case .legal:
            LegalView(viewModel: LegalViewModel(path: $flowState.path))
        case .addCredentials:
            AddCredentialsView(viewModel: AddCredentialsViewModel(path: $flowState.path))
        case .logs:
            LogsView(viewModel: LogsViewModel(path: $flowState.path))
        case .acknowledgments:
            AcknowledgmentsView(viewModel: AcknowledgeMentsViewModel(path: $flowState.path))
        case let .licence(details: details):
            LicenseView(viewModel: LicenseViewModel(path: $flowState.path, details: details))
        }
    }
}
