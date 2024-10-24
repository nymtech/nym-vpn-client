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
    // MARK: - Title -
    public struct Title {
        public struct Large {
            public static var primary: NymTextStyle {
                NymTextStyle(nymFont: .lato(size: 22, weight: .regular))
            }
        }

        public struct Medium {
            public static var primary: NymTextStyle {
                NymTextStyle(nymFont: .lato(size: 16, weight: .semibold), kerning: 0.15)
            }
        }
    }

    public struct Headline {
        public struct Small {
            public static var primary: NymTextStyle {
                NymTextStyle(nymFont: .lato(size: 24, weight: .regular))
            }
        }
    }

    // MARK: - Label -
    public struct Label {
        public struct Huge {
            public static var bold: NymTextStyle {
                NymTextStyle(nymFont: .lato(size: 18, weight: .bold))
            }
        }

        public struct Large {
            public static var bold: NymTextStyle {
                NymTextStyle(nymFont: .lato(size: 14, weight: .bold), kerning: 0.1)
            }
        }

        public struct Medium {
            public static var primary: NymTextStyle {
                NymTextStyle(nymFont: .lato(size: 12, weight: .medium), kerning: 0.5)
            }
        }

        public struct Small {
            public static var primary: NymTextStyle {
                NymTextStyle(nymFont: .lato(size: 11, weight: .medium), kerning: 0.5)
            }
        }
    }

    // MARK: - Body -
    public struct Body {
        public struct Large {
            public static var semibold: NymTextStyle {
                NymTextStyle(nymFont: .lato(size: 16, weight: .semibold), kerning: 0.5)
            }

            public static var regular: NymTextStyle {
                NymTextStyle(nymFont: .lato(size: 16, weight: .regular), kerning: 0.5)
            }
        }

        public struct Medium {
            public static var regular: NymTextStyle {
                NymTextStyle(nymFont: .lato(size: 14, weight: .regular), kerning: 0.25)
            }
        }

        public struct Small {
            public static var primary: NymTextStyle {
                NymTextStyle(nymFont: .lato(size: 12, weight: .regular), kerning: 0.4)
            }
        }
    }
}
