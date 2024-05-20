import SwiftUI
import Theme
import UIComponents

struct SurveySuccessViewModel {
    let title = "feedback".localizedString
    let copyText = "feedback.survey.success.text".localizedString

    @Binding var path: NavigationPath

    init(path: Binding<NavigationPath>) {
        _path = path
    }
}

// MARK: - Navigation -
extension SurveySuccessViewModel {
    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
        if !path.isEmpty { path.removeLast() }
    }
}
