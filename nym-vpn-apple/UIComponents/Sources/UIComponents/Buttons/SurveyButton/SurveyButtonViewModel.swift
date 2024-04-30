import SwiftUI
import Theme

public final class SurveyButtonViewModel: ObservableObject {
    public enum ButtonType: Int, CaseIterable, Identifiable {
        case notAtAll
        case notLikely
        case likely
        case veryLikely

        public var id: Self {
            self
        }

        public var title: String {
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
    @Published var hasError: Bool
    @Binding var selectedType: ButtonType?

    public init(type: ButtonType, hasError: Bool, selectedType: Binding<ButtonType?>) {
        self.type = type
        self.hasError = hasError
        self._selectedType = selectedType

        if hasError {
            removeError()
        }
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
        if hasError {
            return NymColor.sysError
        } else {
            return isSelected ? NymColor.primaryOrange : .clear
        }
    }

    func removeError() {
        Timer.scheduledTimer(withTimeInterval: 5.0, repeats: false) { [weak self] _ in
            self?.hasError = false
        }
    }
}
