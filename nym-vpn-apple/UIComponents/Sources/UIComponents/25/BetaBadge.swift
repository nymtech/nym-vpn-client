import SwiftUI
import Theme

/// Small green-outlined pill that labels a feature as Beta.
/// Pass `action` to make it tappable.
public struct BetaBadge: View {
    private let text: String
    private let action: (() -> Void)?

    @State private var isHovered = false

    public init(text: String = "general.beta".localizedString, action: (() -> Void)? = nil) {
        self.text = text
        self.action = action
    }

    public var body: some View {
        if let action {
            Button(action: action) {
                label
            }
            .buttonStyle(.plain)
#if os(macOS)
            .onHover { isHovered = $0 }
            .opacity(isHovered ? 0.7 : 1)
#endif
            .accessibilityAddTraits([.isButton])
        } else {
            label
        }
    }

    private var label: some View {
        Text(text)
            .nymTextStyle(.bodySmallBold)
            .foregroundStyle(Color.Nym.primary)
            .padding(.horizontal, 10)
            .padding(.vertical, 3)
            .overlay {
                Capsule()
                    .stroke(Color.Nym.primary, lineWidth: 1)
            }
            .contentShape(Capsule())
    }
}
