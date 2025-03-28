import SwiftUI
import Theme

public struct SettingButtonViewModel {
    let title: String
    let subtitle: String?
    let isSelected: Bool

    public init(title: String, subtitle: String? = nil, isSelected: Bool) {
        self.title = title
        self.subtitle = subtitle
        self.isSelected = isSelected
    }

    var selectionStrokeColor: Color {
        isSelected ? NymColor.accent : .clear
    }

    var selectionImageName: String {
        isSelected ? "circleSelected" : "circle"
    }

    var selectionImageColor: Color {
        isSelected ? NymColor.accent : NymColor.networkButtonCircle
    }
}
