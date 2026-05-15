import SwiftUI
import Theme

extension View {
    public func nymText(color: Color, style: NymTextStyle) -> some View {
        self
            .foregroundStyle(color)
            .textStyle(style)
    }
}
