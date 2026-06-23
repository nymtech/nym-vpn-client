import CoreFoundation
import UIComponents

enum AuthLayout {
    static let stackSpacing: CGFloat = {
#if os(iOS)
        NymSpacing.component
#else
        NymSpacing.section
#endif
    }()

    static let verticalPadding: CGFloat = NymSpacing.large

    /// Tighter vertical rhythm for the account-creation carousel inside a fixed-height drawer.
    static let carouselStackSpacing: CGFloat = NymSpacing.medium
    static let carouselTitleTopMinSpacing: CGFloat = NymSpacing.small

    static let passphraseTextAreaHeight: CGFloat = {
#if os(iOS)
        96
#else
        127
#endif
    }()
}
