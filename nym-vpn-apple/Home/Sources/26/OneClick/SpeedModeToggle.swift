import SwiftUI
import Theme
import UIComponents

struct SpeedModeToggle: View {
    let isFast: Bool
    let onToggle: (Bool) -> Void

    @Environment(\.colorScheme)
    private var colorScheme
    @Environment(\.accessibilityVoiceOverEnabled)
    private var voiceOverEnabled
    @State private var isPillHovered = false

    var body: some View {
        HStack(spacing: NymSpacing.large) {
            Button { onToggle(true) } label: {
                Text("oneClick.speedMode.fast".localizedString)
                    .nymTextStyle(isFast ? .bodyDefaultBold : .bodyDefault)
                    .foregroundStyle(isFast ? Color.Nym.primary : Color.Nym.textSecondary)
                    .frame(maxWidth: .infinity, alignment: .trailing)
            }
            .buttonStyle(.plain)
            .focusEffectDisabled(!voiceOverEnabled)
#if os(macOS)
            .focusable(voiceOverEnabled)
#endif
            .animation(.easeInOut(duration: Constants.toggleDuration), value: isFast)

            Button { onToggle(!isFast) } label: {
                ZStack {
                    Capsule()
                        .fill(Color.Nym.background)
                    ZStack {
                        Circle()
                            .fill(thumbFill)
                            .overlay(
                                colorScheme == .light
                                    ? Circle().stroke(Color.Nym.divider, lineWidth: 1)
                                    : nil
                            )
                        thumbIcon
                            .accessibilityHidden(true)
                    }
                    .shadow(
                        color: isFast ? Color.Nym.primary.opacity(1.0) : Color.clear,
                        radius: Constants.thumbShadowRadius,
                        x: 0,
                        y: 0
                    )
                    .clipShape(Circle())
                    .frame(width: Constants.thumbSize, height: Constants.thumbSize)
                    .offset(x: isFast ? -Constants.thumbOffset : Constants.thumbOffset)
                    .animation(.easeInOut(duration: Constants.toggleDuration), value: isFast)
                    if isPillHovered {
                        Capsule()
                            .fill(Color.Nym.white8)
                    }
                }
                .frame(width: Constants.pillWidth, height: Constants.pillHeight)
            }
            .buttonStyle(.plain)
            .focusEffectDisabled(!voiceOverEnabled)
#if os(macOS)
            .focusable(voiceOverEnabled)
#endif
            .onHover { isPillHovered = $0 }
            .accessibilityLabel("oneClick.speedMode.accessibilityLabel".localizedString)
            .accessibilityValue(
                isFast
                    ? "oneClick.speedMode.fast".localizedString
                    : "oneClick.speedMode.anonymous".localizedString
            )
            .accessibilityHint("oneClick.speedMode.accessibilityHint".localizedString)

            Button { onToggle(false) } label: {
                Text("oneClick.speedMode.anonymous".localizedString)
                    .nymTextStyle(!isFast ? .bodyDefaultBold : .bodyDefault)
                    .foregroundStyle(!isFast ? Color.Nym.primary : Color.Nym.textSecondary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
            .buttonStyle(.plain)
            .focusEffectDisabled(!voiceOverEnabled)
#if os(macOS)
            .focusable(voiceOverEnabled)
#endif
            .animation(.easeInOut(duration: Constants.toggleDuration), value: isFast)
        }
    }
}

private extension SpeedModeToggle {
    @ViewBuilder var thumbIcon: some View {
        Image(systemName: isFast ? "bolt.fill" : "eye.slash.fill")
            .font(.system(size: Constants.thumbIconSize, weight: .semibold))
            .foregroundStyle(Color.Nym.primary)
    }

    var thumbFill: Color {
        switch (isFast, colorScheme) {
        case (true, .dark):
            Color.Nym.background
        case (false, .dark):
            Color.Nym.gray12
        default:
            Color.Nym.white
        }
    }

    enum Constants {
        static let thumbSize: CGFloat = 28
        static let thumbOffset: CGFloat = 20
        static let thumbShadowRadius: CGFloat = 8
        static let thumbIconSize: CGFloat = 14
        static let pillWidth: CGFloat = 80
        static let pillHeight: CGFloat = 44
        static let toggleDuration: Double = 0.2
    }
}
