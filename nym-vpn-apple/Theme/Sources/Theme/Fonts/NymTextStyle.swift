import SwiftUI

public struct NymTextStyle {
    let nymFont: NymFont
    let lineSpacing: CGFloat
    let kerning: CGFloat

    init(nymFont: NymFont, lineSpacing: CGFloat = 0, kerning: CGFloat = 0) {
        self.nymFont = nymFont
        self.lineSpacing = lineSpacing
        self.kerning = kerning
    }
}

// MARK: - Styles -
extension NymTextStyle {
    public struct Headline {
        public struct Large {
            public static var regular: NymTextStyle {
                NymTextStyle(nymFont: .labGrotesqueMono(size: 24, weight: .regular), kerning: 1.2)
            }
        }

        public struct Medium {
            public static var regular: NymTextStyle {
                NymTextStyle(nymFont: .labGrotesqueMono(size: 20, weight: .regular), kerning: 1)
            }
        }

        public struct Small {
            public static var regular: NymTextStyle {
                NymTextStyle(nymFont: .labGrotesqueMono(size: 16, weight: .regular), kerning: 0.8)
            }
        }
    }

    public struct Body {
        public struct Large {
            public static var regular: NymTextStyle {
                NymTextStyle(nymFont: .labGrotesque(size: 16, weight: .regular), kerning: 0.32)
            }
        }

        public struct Medium {
            public static var regular: NymTextStyle {
                NymTextStyle(nymFont: .labGrotesque(size: 14, weight: .regular), kerning: 0.28)
            }
        }

        public struct Small {
            public static var regular: NymTextStyle {
                NymTextStyle(nymFont: .labGrotesque(size: 12, weight: .regular), kerning: 0.24)
            }
        }
    }
}
