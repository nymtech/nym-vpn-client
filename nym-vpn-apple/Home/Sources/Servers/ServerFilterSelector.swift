import SwiftUI
import Theme
import UIComponents

/// Favorites / Recent / All servers segmented pill shown above the gateways list.
struct ServerFilterSelector: View {
    @Binding var selection: ServerFilter

    @Environment(\.accessibilityVoiceOverEnabled)
    private var voiceOverEnabled

    @State private var hovered: ServerFilter?

    var body: some View {
        HStack(spacing: Constants.segmentGap) {
            ForEach(ServerFilter.allCases, id: \.self) { filter in
                segmentButton(for: filter)
            }
        }
        .padding(Constants.outerPadding)
        .background(
            Capsule().fill(Color.Nym.surfacePressed)
        )
        .frame(height: Constants.pillHeight)
        .accessibilityElement(children: .contain)
    }
}

private extension ServerFilterSelector {
    @ViewBuilder
    func segmentButton(for filter: ServerFilter) -> some View {
        let isSelected = filter == selection
        let isHovered = hovered == filter
        let tint = tintColor(isSelected: isSelected, isHovered: isHovered)
        Button {
            selection = filter
        } label: {
            HStack(spacing: Constants.iconLabelGap) {
                Image(systemName: iconName(for: filter, isSelected: isSelected))
                    .font(.system(size: Constants.iconSize, weight: .semibold))
                    .foregroundStyle(tint)
                    .accessibilityHidden(true)
                Text(filter.localizedTitle)
                    .nymTextStyle(isSelected ? .bodyDefaultBold : .bodyDefault)
                    .foregroundStyle(tint)
                    .lineLimit(1)
            }
            .frame(maxWidth: .infinity)
            .padding(.vertical, Constants.segmentVerticalPadding)
            .background(
                Capsule().fill(isSelected ? Color.Nym.surface : Color.clear)
            )
            .overlay(
                Capsule().strokeBorder(isSelected ? Color.Nym.primary : Color.clear, lineWidth: 1)
            )
            .contentShape(Capsule())
        }
        .buttonStyle(.plain)
        .focusEffectDisabled(!voiceOverEnabled)
#if os(macOS)
        .focusable(voiceOverEnabled)
#endif
        .onHover { hovering in
            hovered = hovering ? filter : (hovered == filter ? nil : hovered)
        }
        .animation(.easeInOut(duration: Constants.animationDuration), value: selection)
        .animation(.easeInOut(duration: Constants.animationDuration), value: hovered)
        .accessibilityLabel(filter.localizedTitle)
        .accessibilityAddTraits(isSelected ? [.isSelected, .isButton] : .isButton)
    }

    func iconName(for filter: ServerFilter, isSelected: Bool) -> String {
        if filter == .favorites {
            return isSelected ? "star.fill" : "star"
        }
        return filter.systemImageName
    }

    func tintColor(isSelected: Bool, isHovered: Bool) -> Color {
        if isHovered {
            return Color.Nym.primaryHover
        }
        return isSelected ? Color.Nym.primary : Color.Nym.textSecondary
    }

    enum Constants {
        static let pillHeight: CGFloat = 40
        static let outerPadding: CGFloat = 2
        static let segmentGap: CGFloat = 0
        static let iconLabelGap: CGFloat = 6
        static let iconSize: CGFloat = 13
        static let segmentVerticalPadding: CGFloat = 6
        static let animationDuration: Double = 0.2
    }
}
