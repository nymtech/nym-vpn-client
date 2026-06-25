import SwiftUI
import Theme

public struct StrokeBorderView<Content: View>: View {
    @ViewBuilder private let content: Content
    private let strokeTitle: String
    private let strokeColor: Color
    private let backgroundColor: Color
    private let backgrounsColorHover: Color

    @Binding private var isHovered: Bool

    public init(
        strokeTitle: String,
        isHovered: Binding<Bool>,
        strokeColor: Color = NymColor.gray2,
        backgroundColor: Color = NymColor.background,
        backgroundColorHover: Color = NymColor.backgroundHover,
        @ViewBuilder content: () -> Content
    ) {
        self.strokeTitle = strokeTitle
        self.strokeColor = strokeColor
        self.backgroundColor = backgroundColor
        self.backgrounsColorHover = backgroundColorHover
        self.content = content()
        _isHovered = isHovered
    }

    public var body: some View {
        VStack(alignment: .leading) {
            content
        }
        .frame(height: 56)
        .background(isHovered ? backgrounsColorHover : backgroundColor)
        .cornerRadius(8)
        .overlay {
            RoundedRectangle(cornerRadius: 8)
                .inset(by: 0.5)
                .stroke(strokeColor.opacity(isHovered ? 0.7 : 1), lineWidth: 1)
        }
        .overlay(alignment: .topLeading) {
            HStack {
                Text(strokeTitle)
                    .foregroundStyle(NymColor.primary)
                    .textStyle(.Body.Small.regular)
                    .padding(.horizontal, 4)
                    .background(isHovered ? backgrounsColorHover : backgroundColor)
                    .offset(x: 8, y: -7)
                Spacer()
            }
            .frame(maxWidth: .infinity)
            .accessibilityHidden(true)
        }
    }
}
