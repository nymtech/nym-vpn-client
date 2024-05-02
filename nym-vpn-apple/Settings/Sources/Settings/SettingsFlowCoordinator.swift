import SwiftUI
import AppSettings

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
        case .survey:
            SurveyView(viewModel: SurveyViewModel(path: $flowState.path))
        case .surveySuccess:
            SurveySuccessView(viewModel: SurveySuccessViewModel(path: $flowState.path))
        case .addCredentials:
            AddCredentialsView(viewModel: AddCredentialsViewModel(path: $flowState.path))
        }
    }
}
