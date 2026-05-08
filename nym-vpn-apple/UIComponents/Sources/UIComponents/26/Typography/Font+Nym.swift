import SwiftUI

public extension Font {
    enum Nym {
        public static let titleScreen     = Font.custom("LabGrotesque-Bold", size: 18, relativeTo: .title3)
        public static let titleSection    = Font.custom("LabGrotesque-Bold", size: 16, relativeTo: .headline)
        public static let titleSmall      = Font.custom("LabGrotesque-Bold", size: 14, relativeTo: .subheadline)

        public static let bodyLarge       = Font.custom("LabGrotesque-Regular", size: 16, relativeTo: .body)
        public static let bodyDefault     = Font.custom("LabGrotesque-Regular", size: 14, relativeTo: .callout)
        public static let bodyDefaultBold = Font.custom("LabGrotesque-Bold", size: 14, relativeTo: .callout)
        public static let bodySmall       = Font.custom("LabGrotesque-Regular", size: 12, relativeTo: .caption)
        public static let bodySmallBold   = Font.custom("LabGrotesque-Bold", size: 12, relativeTo: .caption)

        public static let subheading      = Font.custom("LabGrotesque-Regular", size: 12, relativeTo: .caption)
    }
}
