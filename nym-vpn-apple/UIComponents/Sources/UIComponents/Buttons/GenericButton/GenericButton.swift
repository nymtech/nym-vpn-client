import SwiftUI
import Theme

public struct GenericButton: View {
    private let title: String
    private let borderOnly: Bool
    private let mainColor: Color
    private let height: CGFloat

    @State private var isHovered = false

    public init(title: String, borderOnly: Bool = false, mainColor: Color = NymColor.accent, height: CGFloat = 56) {
        self.title = title
        self.borderOnly = borderOnly
        self.mainColor = mainColor
        self.height = height
    }

    public var body: some View {
        HStack {
            Text(title)
                .foregroundStyle(borderOnly ? mainColor : NymColor.connectTitle)
                .textStyle(.Headline.Small.regular)
                .padding(.vertical, 10)
        }
        .frame(maxWidth: .infinity, minHeight: height, maxHeight: height)
        .background {
            borderOnly ? .clear : mainColor.opacity(isHovered ? 0.7 : 1)
        }
        .overlay(
            RoundedRectangle(cornerRadius: 8)
                .stroke(mainColor, lineWidth: borderOnly ? 1 : 0)
        )
        .contentShape(RoundedRectangle(cornerRadius: 8))
        .cornerRadius(8)
        .onHover { newValue in
            isHovered = newValue
        }
    }
}
