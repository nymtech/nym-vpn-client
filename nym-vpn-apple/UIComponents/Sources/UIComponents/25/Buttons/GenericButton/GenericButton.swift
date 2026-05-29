import SwiftUI
import Theme

public struct GenericButton: View {
    public enum Style {
        case normal
        case borderless
        case accentBorderOnly
        case primaryBorderOnly
        case textOnly
        case destructive

        func backgroundColor(isDisabled: Bool) -> Color {
            switch self {
            case .normal:
                isDisabled ? Color.Nym.textSecondary : Color.Nym.primary
            case .accentBorderOnly, .textOnly, .primaryBorderOnly, .borderless:
                .clear
            case .destructive:
                Color.Nym.error.opacity(0.1)
            }
        }

        var imageForegroundColor: Color {
            switch self {
            case .normal, .borderless:
                Color.Nym.primaryText
            case .accentBorderOnly, .textOnly:
                Color.Nym.primary
            case .primaryBorderOnly, .destructive:
                Color.Nym.textPrimary
            }
        }

        func textTitleColor(isDisabled: Bool) -> Color {
            switch self {
            case .normal:
                Color.Nym.primaryText
            case .borderless:
                Color.Nym.textPrimary
            case .accentBorderOnly:
                Color.Nym.primary
            case .textOnly, .primaryBorderOnly, .destructive:
                isDisabled ? Color.Nym.textSecondary : Color.Nym.textPrimary
            }
        }

        var strokeLineWidth: CGFloat {
            switch self {
            case .normal, .textOnly, .borderless:
                0
            case .accentBorderOnly, .primaryBorderOnly, .destructive:
                1
            }
        }

        func strokeColor(isDisabled: Bool) -> Color {
            switch self {
            case .normal, .textOnly, .borderless:
                .clear
            case .accentBorderOnly:
                Color.Nym.primary
            case .primaryBorderOnly:
                isDisabled ? Color.Nym.textSecondary : Color.Nym.textPrimary
            case .destructive:
                Color.Nym.error
            }
        }
    }

    private let title: String
    private let titleColor: Color?
    private let style: Style
    private let height: CGFloat
    private let isWidthExpanded: Bool
    private let isExternalLink: Bool
    private let systemImageName: String?
    private let isSystemImageFlipped: Bool

    @State private var isHovered = false
    @Binding private var isLoading: Bool
    @Binding private var isDisabled: Bool

    public init(
        title: String,
        titleColor: Color? = nil,
        style: Style = .normal,
        height: CGFloat = 56,
        isDisabled: Binding<Bool> = .constant(false),
        isLoading: Binding<Bool> = .constant(false),
        isWidthExpanded: Bool = true,
        isExternalLink: Bool = false,
        systemImageName: String? = nil,
        isSystemImageFlipped: Bool = false
    ) {
        self.title = title
        self.titleColor = titleColor
        self.style = style
        self.height = height

        self.isWidthExpanded = isWidthExpanded
        self.isExternalLink = isExternalLink
        self.systemImageName = systemImageName
        self.isSystemImageFlipped = isSystemImageFlipped
        _isLoading = isLoading
        _isDisabled = isDisabled
    }

    public var body: some View {
        HStack {
            if isLoading {
                ProgressView()
                    .colorScheme(.light)
            } else {
                if let systemImageName {
                    Image(systemName: systemImageName)
                        .resizable()
                        .scaledToFit()
                        .frame(width: 24, height: 24)
                        .padding(.horizontal, 8)
                        .scaleEffect(x: isSystemImageFlipped ? -1 : 1, y: 1)
                        .foregroundStyle(style.imageForegroundColor)
                }

                Text(title)
                    .foregroundStyle(titleColor ?? style.textTitleColor(isDisabled: isDisabled))
                    .textStyle(.Headline.Small.regular)
                    .minimumScaleFactor(0.8)

                if isExternalLink {
                    ExternalLinkImage(color: titleColor ?? style.textTitleColor(isDisabled: isDisabled))
                }
            }
        }
        .padding(EdgeInsets(top: 12, leading: 16, bottom: 8, trailing: 16))
        .accessibilityLabel(title)
        .accessibilityAddTraits([.isButton])
        .frame(maxWidth: isWidthExpanded ? .infinity : nil)
        .frame(height: height)
        .background {
            style.backgroundColor(isDisabled: isDisabled).opacity(isHovered ? 0.7 : 1)
        }
        .overlay(
            RoundedRectangle(cornerRadius: 8)
                .stroke(style.strokeColor(isDisabled: isDisabled), lineWidth: style.strokeLineWidth)
        )
        .contentShape(RoundedRectangle(cornerRadius: 8))
        .cornerRadius(8)
        .onHover { newValue in
            isHovered = newValue
        }
    }
}
