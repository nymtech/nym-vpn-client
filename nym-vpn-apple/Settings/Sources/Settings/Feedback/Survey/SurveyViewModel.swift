import SwiftUI
import UIComponents

struct SurveyViewModel {
    let title = "feedback".localizedString
    let introText = "feedback.survey.intro".localizedString
    let recommendQuestionText = "feedback.survey.howLikelyRecommend".localizedString
    let provideFeedbackQuestionText = "feedback.survey.askFeedback".localizedString
    let yourFeedbackPlacholderText = "feedback.survey.yourFeedback".localizedString

    @Binding var path: NavigationPath
    @State var feedbackText = ""
    var selectedRecommendation: SurveyButtonViewModel.ButtonType?

    init(path: Binding<NavigationPath>) {
        _path = path
    }
}

// MARK: - Navigation -
extension SurveyViewModel {
    func navigateBack() {
        path.removeLast()
    }
}
