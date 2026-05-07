import SwiftUI
import Theme

public struct NymButton: View {
    public enum Style {
        case primary
        case secondary
        case textOnly
        case destructive
        case connecting
        case connected

        var backgroundColor: Color {
            switch self {
            case .primary:
                .Nym.primary
            case .connecting:
                .Nym.gray1
            case .secondary,
                 .textOnly,
                 .destructive,
                 .connected:
                .clear
            }
        }

        var foregroundColor: Color {
            switch self {
            case .primary:
                .Nym.black
            case .connecting:
                .Nym.gray12
            case .secondary,
                 .textOnly:
                .Nym.primary
            case .connected:
                .Nym.textPrimary
            case .destructive:
                .Nym.error
            }
        }

        var borderColor: Color {
            switch self {
            case .primary, .textOnly, .connecting:
                .clear
            case .secondary:
                .Nym.primary
            case .destructive:
                .Nym.error
            case .connected:
                .Nym.gray2
            }
        }

        var borderWidth: CGFloat {
            switch self {
            case .primary, .textOnly, .connecting:
                0
            case .secondary, .destructive, .connected:
                1
            }
        }
    }

    private let label: String
    private let style: Style
    private let cornerRadius: CGFloat
    private let foregroundColorOverride: Color?
    private let borderColorOverride: Color?
    private let trailingSystemImage: String?
    private let action: () -> Void

    @Environment(\.isEnabled)
    private var isEnabled
    @Environment(\.accessibilityVoiceOverEnabled)
    private var voiceOverEnabled
    @State private var isHovered = false
    @State private var isDisabled: Bool

    public init(
        _ label: String,
        style: Style = .primary,
        cornerRadius: CGFloat = 8,
        foregroundColor: Color? = nil,
        borderColor: Color? = nil,
        trailingSystemImage: String? = nil,
        isDisabled: Bool = false,
        action: @escaping () -> Void
    ) {
        self.label = label
        self.style = style
        self.cornerRadius = cornerRadius
        self.foregroundColorOverride = foregroundColor
        self.borderColorOverride = borderColor
        self.trailingSystemImage = trailingSystemImage
        self.action = action
        _isDisabled = State(initialValue: isDisabled)
    }

    public var body: some View {
        Button(action: action) {
            buttonContent
                .frame(maxWidth: .infinity)
                .frame(height: Constants.height)
                .background(effectiveBackground)
                .overlay(
                    RoundedRectangle(cornerRadius: cornerRadius)
                        .stroke(effectiveBorderColor, lineWidth: style.borderWidth)
                )
                .clipShape(RoundedRectangle(cornerRadius: cornerRadius))
                .contentShape(RoundedRectangle(cornerRadius: cornerRadius))
        }
        .buttonStyle(.plain)
        .disabled(isDisabled)
        .onHover { isHovered = $0 }
        .focusEffectDisabled(!voiceOverEnabled)
#if os(macOS)
        .focusable(voiceOverEnabled)
#endif
    }
}

private extension NymButton {
    enum Constants {
        static let height: CGFloat = 45
    }

    @ViewBuilder var buttonContent: some View {
        if let trailingSystemImage {
            HStack(spacing: NymSpacing.small) {
                Spacer(minLength: 0)
                Text(verbatim: label)
                    .nymTextStyle(.titleSmall)
                    .foregroundStyle(effectiveForeground)
                Image(systemName: trailingSystemImage)
                    .font(.system(size: 18))
                    .foregroundStyle(effectiveForeground)
                Spacer(minLength: 0)
            }
        } else {
            Text(verbatim: label)
                .nymTextStyle(.titleSmall)
                .foregroundStyle(effectiveForeground)
        }
    }

    var effectiveForeground: Color {
        if !isEnabled && style == .connecting {
            return .Nym.gray12
        }
        return isEnabled ? (foregroundColorOverride ?? style.foregroundColor) : .Nym.textDisabled
    }

    var effectiveBackground: Color {
        guard isEnabled else {
            switch style {
            case .connecting:
                return .Nym.gray1
            case .primary:
                return .Nym.textDisabled.opacity(0.3)
            default:
                return .clear
            }
        }
        return isHovered ? style.backgroundColor.opacity(0.75) : style.backgroundColor
    }

    var effectiveBorderColor: Color {
        if !isEnabled && style == .connecting {
            return .clear
        }
        return isEnabled ? (borderColorOverride ?? style.borderColor) : .Nym.textDisabled.opacity(0.3)
    }
}

#if DEBUG
#Preview {
    VStack(spacing: NymSpacing.medium) {
        NymButton("Connect", style: .primary, cornerRadius: 28) {}
        NymButton("Learn more", style: .secondary) {}
        NymButton("Skip", style: .textOnly) {}
        NymButton("Delete account", style: .destructive) {}
        NymButton("Disabled", style: .primary, isDisabled: true) {}
    }
    .padding(NymSpacing.section)
    .background(Color.Nym.background)
}
#endif
