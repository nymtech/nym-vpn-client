import SwiftUI
import Theme

public struct GenericButton: View {
    private let title: String
    private let borderOnly: Bool
    private let mainColor: Color
    private let height: CGFloat
    private let isWidthExpanded: Bool

    @State private var isHovered = false

    public init(
        title: String,
        borderOnly: Bool = false,
        mainColor: Color = NymColor.accent,
        height: CGFloat = 56,
        isWidthExpanded: Bool = true
    ) {
        self.title = title
        self.borderOnly = borderOnly
        self.mainColor = mainColor
        self.height = height
        self.isWidthExpanded = isWidthExpanded
    }

    public var body: some View {
        HStack {
            Text(title)
                .foregroundStyle(borderOnly ? mainColor : NymColor.black)
                .textStyle(.Headline.Small.regular)
                .padding(EdgeInsets(top: 10, leading: 16, bottom: 10, trailing: 16))
        }
        .frame(maxWidth: isWidthExpanded ? .infinity : nil)
        .frame(height: height)
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
