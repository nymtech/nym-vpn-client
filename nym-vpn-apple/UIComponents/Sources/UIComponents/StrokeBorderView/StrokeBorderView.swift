import SwiftUI
import Theme

public struct StrokeBorderView<Content: View>: View {
    @ViewBuilder private let content: Content
    private let strokeTitle: String
    private let strokeTitleLeftMargin: CGFloat

    @Binding private var isHovered: Bool

    public init(
        strokeTitle: String,
        strokeTitleLeftMargin: CGFloat,
        isHovered: Binding<Bool>,
        @ViewBuilder content: () -> Content
    ) {
        self.strokeTitle = strokeTitle
        self.strokeTitleLeftMargin = strokeTitleLeftMargin
        self.content = content()
        _isHovered = isHovered
    }

    public var body: some View {
        VStack(alignment: .leading) {
            content
        }
        .background(isHovered ? NymColor.backgroundHover : NymColor.background)
        .frame(height: 56)
        .cornerRadius(8)
        .overlay {
            RoundedRectangle(cornerRadius: 8)
                .inset(by: 0.5)
                .stroke(NymColor.gray2.opacity(isHovered ? 0.7 : 1), lineWidth: 1)
        }
        .overlay(alignment: .topLeading) {
            Text(strokeTitle)
                .foregroundStyle(NymColor.sysOnSurface)
                .textStyle(.BodyLegacy.Small.primary)
                .padding(4)
                .background(NymColor.background, in: RoundedRectangle(cornerRadius: 8))
                .position(x: strokeTitleLeftMargin, y: 0)
        }
    }
}
