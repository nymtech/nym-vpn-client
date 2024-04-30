import SwiftUI
import SentryManager
import Theme
import UIComponents

final class SurveyViewModel: ObservableObject {
    let title = "feedback".localizedString
    let introText = "feedback.survey.intro".localizedString
    let recommendQuestionText = "feedback.survey.howLikelyRecommend".localizedString
    let provideFeedbackQuestionText = "feedback.survey.askFeedback".localizedString
    let yourFeedbackPlacholderText = "feedback.survey.yourFeedback".localizedString
    let submitButtonTitle = "feedback.survey.submit".localizedString

    var sentryManager: SentryManager

    @Binding var path: NavigationPath

    @Published var feedbackText = ""
    @Published var error = SurveyError.noError
    @Published var selectedRecommendation: SurveyButtonViewModel.ButtonType?

    var textFieldStrokeColor: Color {
        error == .missingFeedback ? NymColor.sysError : NymColor.sysOutlineVariant
    }

    var surveyButtonShouldShowError: Bool {
        error == .missingRecommendation && selectedRecommendation == nil
    }

    init(path: Binding<NavigationPath>, sentryManager: SentryManager = SentryManager.shared) {
        _path = path
        self.sentryManager = sentryManager
    }

    func submit() {
        if selectedRecommendation == nil {
            error = .missingRecommendation
        } else if feedbackText.isEmpty {
            error = .missingFeedback
        } else {
            error = .noError
            guard let selectedRecommendation = selectedRecommendation
            else {
                error = .missingRecommendation
                return
            }
            sentryManager.submitFeedback(recommendation: selectedRecommendation.title, message: feedbackText)
            navigateToSurveySuccess()
        }
    }
}

// MARK: - Navigation -
extension SurveyViewModel {
    func navigateBack() {
        path.removeLast()
    }

    func navigateToSurveySuccess() {
        path.append(SettingsLink.surveySuccess)
    }
}
