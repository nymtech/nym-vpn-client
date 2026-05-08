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

    static let passphraseTextAreaHeight: CGFloat = {
#if os(iOS)
        96
#else
        127
#endif
    }()
}
