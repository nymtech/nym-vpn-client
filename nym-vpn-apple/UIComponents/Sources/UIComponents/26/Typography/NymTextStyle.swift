import SwiftUI

public enum Nym {}

public extension Nym {
    struct TextStyle {
        public let font: Font
        public let tracking: CGFloat
        public let lineSpacing: CGFloat

        public init(font: Font, tracking: CGFloat = 0, lineSpacing: CGFloat = 0) {
            self.font = font
            self.tracking = tracking
            self.lineSpacing = lineSpacing
        }
    }
}

public extension Nym.TextStyle {
    static let titleScreen     = Nym.TextStyle(font: .Nym.titleScreen, tracking: -1)
    static let titleSection    = Nym.TextStyle(font: .Nym.titleSection, tracking: 1)
    static let titleSmall      = Nym.TextStyle(font: .Nym.titleSmall, tracking: 1)
    static let bodyLarge       = Nym.TextStyle(font: .Nym.bodyLarge, tracking: -0.5, lineSpacing: 4)
    static let bodyDefault     = Nym.TextStyle(font: .Nym.bodyDefault, tracking: 1, lineSpacing: 2)
    static let bodyDefaultBold = Nym.TextStyle(font: .Nym.bodyDefaultBold, tracking: 0.5)
    static let bodySmall       = Nym.TextStyle(font: .Nym.bodySmall, tracking: 1.5, lineSpacing: 2)
    static let bodySmallBold   = Nym.TextStyle(font: .Nym.bodySmallBold, tracking: 0.5)
    static let subheading      = Nym.TextStyle(font: .Nym.subheading, tracking: 2)
}

public extension View {
    func nymTextStyle(_ style: Nym.TextStyle) -> some View {
        font(style.font)
            .tracking(style.tracking)
            .lineSpacing(style.lineSpacing)
    }
}
