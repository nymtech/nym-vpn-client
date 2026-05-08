import SwiftUI
import Theme

public struct NymBackButton: View {
    private let action: () -> Void

    @Environment(\.accessibilityVoiceOverEnabled)
    private var voiceOverEnabled

    public init(action: @escaping () -> Void) {
        self.action = action
    }

    public var body: some View {
        Button(action: action) {
            Image(systemName: "chevron.left")
                .font(.system(size: 18, weight: .medium))
                .foregroundStyle(Color.Nym.textPrimary)
                .frame(width: 32, height: 32)
                .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .focusEffectDisabled(!voiceOverEnabled)
#if os(macOS)
        .focusable(voiceOverEnabled)
#endif
        .accessibilityLabel("back".localizedString)
        .accessibilityAddTraits(.isButton)
    }
}
