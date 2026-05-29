import SwiftUI
import Theme

/// A button that renders a Nym design-system image with a HIG-compliant tap
/// target and, on macOS, a circular hover highlight centred on the image.
///
/// ```swift
/// ImageButton(
///     systemImageName: "gear",
///     imageSize: 36,
///     accessibilityLabel: "Settings"
/// ) {
///     viewModel.trailingButtonTapped()
/// }
/// ```
public struct ImageButton: View {
    private let imageName: String?
    private let systemImageName: String?
    private let imageSize: CGFloat
    private let renderSize: CGFloat?
    private let layoutSize: CGFloat?
    private let accessibilityLabel: String
    private let accessibilityHint: String?
    private let action: () -> Void

    @Environment(\.colorScheme)
    private var colorScheme
    @Environment(\.accessibilityVoiceOverEnabled)
    private var voiceOverEnabled
    @State private var isHovered = false

    public init(
        imageName: String,
        imageSize: CGFloat,
        accessibilityLabel: String,
        renderSize: CGFloat? = nil,
        layoutSize: CGFloat? = nil,
        accessibilityHint: String? = nil,
        action: @escaping () -> Void
    ) {
        self.imageName = imageName
        self.systemImageName = nil
        self.imageSize = imageSize
        self.renderSize = renderSize
        self.layoutSize = layoutSize
        self.accessibilityLabel = accessibilityLabel
        self.accessibilityHint = accessibilityHint
        self.action = action
    }

    public init(
        systemImageName: String,
        imageSize: CGFloat,
        accessibilityLabel: String,
        renderSize: CGFloat? = nil,
        layoutSize: CGFloat? = nil,
        accessibilityHint: String? = nil,
        action: @escaping () -> Void
    ) {
        self.imageName = nil
        self.systemImageName = systemImageName
        self.imageSize = imageSize
        self.renderSize = renderSize
        self.layoutSize = layoutSize
        self.accessibilityLabel = accessibilityLabel
        self.accessibilityHint = accessibilityHint
        self.action = action
    }

    public var body: some View {
        Button(action: action) {
            image
                .frame(width: renderSize ?? imageSize, height: renderSize ?? imageSize)
                .frame(width: imageSize, height: imageSize)
                .background(hoverCircle)
                .frame(
                    minWidth: AccessibilityConstants.MinTapTarget.size,
                    minHeight: layoutSize ?? AccessibilityConstants.MinTapTarget.size
                )
        }
        .buttonStyle(.plain)
        .contentShape(Rectangle().inset(by: tapTargetInset))
        .onHover { isHovered = $0 }
        .focusEffectDisabled(!voiceOverEnabled)
#if os(macOS)
        .focusable(voiceOverEnabled)
#endif
        .accessibilityLabel(accessibilityLabel)
        .accessibilityHintIfPresent(accessibilityHint)
        .accessibilityAddTraits(.isButton)
    }
}

private extension ImageButton {
    enum Constants {
        enum HoverCircle {
            static let expansion: CGFloat = 4
        }
    }

    var hoverColor: Color {
        colorScheme == .dark ? Color.Nym.divider : Color.Nym.surfaceAlt.opacity(0.15)
    }

    var tapTargetInset: CGFloat {
        guard let layoutSize, layoutSize < AccessibilityConstants.MinTapTarget.size else {
            return 0
        }
        return -((AccessibilityConstants.MinTapTarget.size - layoutSize) / 2)
    }

    @ViewBuilder var image: some View {
        if let imageName {
            GenericImage(imageName: imageName)
        } else if let systemImageName {
            GenericImage(systemImageName: systemImageName)
        }
    }

    @ViewBuilder var hoverCircle: some View {
        if isHovered {
            Circle()
                .fill(hoverColor)
                .padding(-Constants.HoverCircle.expansion)
        }
    }
}
