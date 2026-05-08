import SwiftUI

extension View {
    /// Applies `.accessibilityHint` only when `hint` is non-nil, avoiding the
    /// VoiceOver quirk of announcing an empty hint string.
    @ViewBuilder
    func accessibilityHintIfPresent(_ hint: String?) -> some View {
        if let hint {
            accessibilityHint(hint)
        } else {
            self
        }
    }
}
