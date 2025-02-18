import SwiftUI
import Theme

public struct StrokeBorderView<Content: View>: View {
    @ViewBuilder private let content: Content
    private let strokeTitle: String
    private let strokeTitleLeftMargin: CGFloat

    public init(strokeTitle: String, strokeTitleLeftMargin: CGFloat, @ViewBuilder content: () -> Content) {
        self.strokeTitle = strokeTitle
        self.strokeTitleLeftMargin = strokeTitleLeftMargin
        self.content = content()
    }

    public var body: some View {
        VStack(alignment: .leading) {
            content
        }
        .contentShape(
            RoundedRectangle(cornerRadius: 8)
                .inset(by: 0.5)
        )
        .padding(0)
        .frame(height: 56)
        .cornerRadius(8)
        .overlay {
            RoundedRectangle(cornerRadius: 8)
                .inset(by: 0.5)
                .stroke(Color(red: 0.29, green: 0.27, blue: 0.31), lineWidth: 1)
        }
        .overlay(alignment: .topLeading) {
            Text(strokeTitle)
                .foregroundStyle(NymColor.sysOnSurface)
                .textStyle(.BodyLegacy.Small.primary)
                .padding(4)
                .background(NymColor.background)
                .position(x: strokeTitleLeftMargin, y: 0)
        }
    }
}
