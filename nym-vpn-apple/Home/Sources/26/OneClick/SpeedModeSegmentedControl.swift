import SwiftUI
import Theme
import UIComponents

struct SpeedModeSegmentedControl: View {
    let selection: OneClickSpeedMode
    let onSelect: (OneClickSpeedMode) -> Void

    @Environment(\.accessibilityVoiceOverEnabled)
    private var voiceOverEnabled

    @State private var hoveredMode: OneClickSpeedMode?

    var body: some View {
        HStack(spacing: Constants.segmentGap) {
            ForEach(OneClickSpeedMode.allCases, id: \.self) { mode in
                segmentButton(for: mode)
            }
        }
        .padding(Constants.outerPadding)
        .background(
            Capsule().fill(Color.Nym.surfaceSunken)
        )
        .frame(height: Constants.pillHeight)
        .accessibilityElement(children: .contain)
    }
}

private extension SpeedModeSegmentedControl {
    @ViewBuilder
    func segmentButton(for mode: OneClickSpeedMode) -> some View {
        let isSelected = mode == selection
        let isHovered = hoveredMode == mode
        let tint = tintColor(isSelected: isSelected, isHovered: isHovered)
        Button { onSelect(mode) } label: {
            HStack(spacing: Constants.iconLabelGap) {
                icon(for: mode, tint: tint)
                Text(label(for: mode))
                    .nymTextStyle(isSelected ? .bodyDefaultBold : .bodyDefault)
                    .foregroundStyle(tint)
                    .lineLimit(1)
            }
            .frame(maxWidth: .infinity)
            .padding(.vertical, Constants.segmentVerticalPadding)
            .background(
                Capsule()
                    .fill(isSelected ? Color.Nym.surfaceElev : Color.clear)
            )
            .contentShape(Capsule())
        }
        .buttonStyle(.plain)
        .focusEffectDisabled(!voiceOverEnabled)
#if os(macOS)
        .focusable(voiceOverEnabled)
#endif
        .onHover { hovering in
            hoveredMode = hovering ? mode : (hoveredMode == mode ? nil : hoveredMode)
        }
        .animation(.easeInOut(duration: Constants.animationDuration), value: selection)
        .animation(.easeInOut(duration: Constants.animationDuration), value: hoveredMode)
        .accessibilityLabel(label(for: mode))
        .accessibilityAddTraits(isSelected ? [.isSelected, .isButton] : .isButton)
    }

    func tintColor(isSelected: Bool, isHovered: Bool) -> Color {
        if isHovered {
            return Color.Nym.brandPrimaryHover
        }
        return isSelected ? Color.Nym.brandPrimary : Color.Nym.textSecondary
    }

    @ViewBuilder
    func icon(for mode: OneClickSpeedMode, tint: Color) -> some View {
        switch mode {
        case .auto:
            Text("N")
                .font(NymFont.labGrotesque(size: Constants.iconSize, weight: .bold).font)
                .foregroundStyle(tint)
                .frame(width: Constants.iconSize, height: Constants.iconSize)
                .accessibilityHidden(true)
        case .fast:
            Image(systemName: "bolt.fill")
                .font(.system(size: Constants.iconSize, weight: .semibold))
                .foregroundStyle(tint)
                .frame(width: Constants.iconSize, height: Constants.iconSize)
                .accessibilityHidden(true)
        case .anonymous:
            Image(systemName: "eye.slash.fill")
                .font(.system(size: Constants.iconSize, weight: .semibold))
                .foregroundStyle(tint)
                .frame(width: Constants.iconSize, height: Constants.iconSize)
                .accessibilityHidden(true)
        }
    }

    func label(for mode: OneClickSpeedMode) -> String {
        switch mode {
        case .auto:
            "oneClick.speedMode.auto".localizedString
        case .fast:
            "oneClick.speedMode.fast".localizedString
        case .anonymous:
            "oneClick.speedMode.anonymous".localizedString
        }
    }

    enum Constants {
        static let pillHeight: CGFloat = 50
        static let outerPadding: CGFloat = 2
        static let segmentGap: CGFloat = 0
        static let iconLabelGap: CGFloat = 6
        static let iconSize: CGFloat = 16
        static let segmentVerticalPadding: CGFloat = 8
        static let animationDuration: Double = 0.2
    }
}
