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
    /// Breathing room between WaveDots and carousel status copy (login + create flows).
    static let carouselLoaderBottomSpacing: CGFloat = NymSpacing.large
    /// Breathing room between the step progress bar and WaveDots (login + create flows).
    static let carouselLoaderTopSpacing: CGFloat = NymSpacing.section

    /// Shared compact carousel rhythm for registration + login processing drawers.
    static let processingCarouselStackSpacing: CGFloat = NymSpacing.small
    static let processingCarouselVerticalPadding: CGFloat = NymSpacing.medium
    static let processingCarouselTitleSpacing: CGFloat = NymSpacing.small

    /// Reserve measured carousel height only while title/subtitle pairs rotate.
    static func processingCarouselTitleReservedHeight(
        didShowFinalMessage: Bool,
        measuredCarouselTitleHeight: CGFloat
    ) -> CGFloat? {
        guard !didShowFinalMessage, measuredCarouselTitleHeight > 0 else { return nil }
        return measuredCarouselTitleHeight
    }

    static let passphraseTextAreaHeight: CGFloat = {
#if os(iOS)
        96
#else
        127
#endif
    }()
}
