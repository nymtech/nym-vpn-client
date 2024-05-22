import Theme

enum SurveyError {
    case missingRecommendation
    case missingFeedback
    case noError

    var localizedTitle: String? {
        switch self {
        case .missingRecommendation:
            return "feedback.survey.error.selectRecommendation".localizedString
        case .missingFeedback:
            return "feedback.survey.error.provideFeedback".localizedString
        case .noError:
            return nil
        }
    }
}
