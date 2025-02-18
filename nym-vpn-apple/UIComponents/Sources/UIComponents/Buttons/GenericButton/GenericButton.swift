import SwiftUI
import Theme

public struct GenericButton: View {
    private let title: String
    private let borderOnly: Bool
    private let mainColor: Color

    public init(title: String, borderOnly: Bool = false, mainColor: Color = NymColor.primaryOrange) {
        self.title = title
        self.borderOnly = borderOnly
        self.mainColor = mainColor
    }

    public var body: some View {
        HStack {
            Text(title)
                .foregroundStyle(borderOnly ? mainColor : NymColor.connectTitle)
                .textStyle(.LabelLegacy.Huge.bold)
        }
        .frame(maxWidth: .infinity, minHeight: 56, maxHeight: 56)
        .background(borderOnly ? .clear : mainColor)
        .overlay(
            RoundedRectangle(cornerRadius: 8)
                .stroke(mainColor, lineWidth: borderOnly ? 1 : 0)
        )
        .contentShape(RoundedRectangle(cornerRadius: 8))
        .cornerRadius(8)
    }
}
