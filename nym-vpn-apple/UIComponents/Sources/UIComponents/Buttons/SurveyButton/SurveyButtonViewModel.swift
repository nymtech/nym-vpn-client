import SwiftUI
import Theme

public struct SurveyButtonViewModel {
    public enum ButtonType: Int, CaseIterable, Identifiable {
        case notAtAll
        case notLikely
        case likely
        case veryLikely

        public var id: Self {
            self
        }

        var title: String {
            switch self {
            case .notAtAll:
                "feedback.survey.notAtAll".localizedString
            case .notLikely:
                "feedback.survey.notLikely".localizedString
            case .likely:
                "feedback.survey.likely".localizedString
            case .veryLikely:
                "feedback.survey.veryLikely".localizedString
            }
        }
    }

    let type: ButtonType

    @Binding var selectedType: ButtonType?

    public init(type: ButtonType, selectedType: Binding<ButtonType?>) {
        self.type = type
        self._selectedType = selectedType
    }

    private var isSelected: Bool {
        type == selectedType
    }

    var selectionImageName: String {
        isSelected ? "networkSelectedCircle" : "networkCircle"
    }

    var selectionImageColor: Color {
        isSelected ? NymColor.primaryOrange : NymColor.networkButtonCircle
    }

    var selectionStrokeColor: Color {
        isSelected ? NymColor.primaryOrange : .clear
    }
}
