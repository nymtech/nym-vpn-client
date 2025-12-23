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
                isDisabled ? NymColor.gray1 : NymColor.accent
            case .accentBorderOnly, .textOnly, .primaryBorderOnly, .borderless:
                .clear
            case .destructive:
                NymColor.error.opacity(0.1)
            }
        }

        var imageForegroundColor: Color {
            switch self {
            case .normal, .borderless:
                NymColor.black
            case .accentBorderOnly, .textOnly:
                NymColor.accent
            case .primaryBorderOnly, .destructive:
                NymColor.primary
            }
        }

        func textTitleColor(isDisabled: Bool) -> Color {
            switch self {
            case .normal:
                NymColor.black
            case .borderless:
                NymColor.primary
            case .accentBorderOnly:
                NymColor.accent
            case .textOnly, .primaryBorderOnly, .destructive:
                isDisabled ? NymColor.gray1 : NymColor.primary
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
                NymColor.accent
            case .primaryBorderOnly:
                isDisabled ? NymColor.gray1 : NymColor.primary
            case .destructive:
                NymColor.error
            }
        }
    }

    private let title: String
    private let titleColor: Color?
    private let style: Style
    private let height: CGFloat
    private let isDisabled: Bool
    private let isWidthExpanded: Bool
    private let systemImageName: String?
    private let isSystemImageFlipped: Bool

    @State private var isHovered = false
    @Binding private var isLoading: Bool

    public init(
        title: String,
        titleColor: Color? = nil,
        style: Style = .normal,
        height: CGFloat = 56,
        isDisabled: Bool = false,
        isLoading: Binding<Bool> = .constant(false),
        isWidthExpanded: Bool = true,
        systemImageName: String? = nil,
        isSystemImageFlipped: Bool = false
    ) {
        self.title = title
        self.titleColor = titleColor
        self.style = style
        self.height = height
        self.isDisabled = isDisabled
        self.isWidthExpanded = isWidthExpanded
        self.systemImageName = systemImageName
        self.isSystemImageFlipped = isSystemImageFlipped
        _isLoading = isLoading
    }

    public var body: some View {
        HStack {
            if isLoading {
                ProgressView()
                    .progressViewStyle(CircularProgressViewStyle(tint: NymColor.black))
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
