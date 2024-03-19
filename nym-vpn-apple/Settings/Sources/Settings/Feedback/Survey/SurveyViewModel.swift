import SwiftUI
import Theme
import UIComponents

final class SurveyViewModel: ObservableObject {
    let title = "feedback".localizedString
    let introText = "feedback.survey.intro".localizedString
    let recommendQuestionText = "feedback.survey.howLikelyRecommend".localizedString
    let provideFeedbackQuestionText = "feedback.survey.askFeedback".localizedString
    let yourFeedbackPlacholderText = "feedback.survey.yourFeedback".localizedString
    let submitButtonTitle = "feedback.survey.submit".localizedString

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

    init(path: Binding<NavigationPath>) {
        _path = path
    }

    func submit() {
        if selectedRecommendation == nil {
            error = .missingRecommendation
        } else if feedbackText.isEmpty {
            error = .missingFeedback
        } else {
            error = .noError
            // TODO: submit feedback to server
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
