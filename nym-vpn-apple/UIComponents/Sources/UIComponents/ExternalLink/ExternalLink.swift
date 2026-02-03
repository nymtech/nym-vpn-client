import SwiftUI
import Theme

public struct ExternalLink: View {
    private let text: AttributedString
    private let color: Color
    private let style: NymTextStyle

    public var body: some View {
        HStack(spacing: 0) {
            Text(text)
                .nymText(color: color, style: style)
            Spacer()
                .frame(width: 4)
            ExternalLinkImage(color: color)
            Spacer()
        }
    }

    public init(text: AttributedString, color: Color, style: NymTextStyle) {
        self.text = text
        self.color = color
        self.style = style
    }
}
