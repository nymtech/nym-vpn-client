import SwiftUI
import Theme

public struct GenericButton: View {
    public enum Style {
        case normal
        case accentBorderOnly
        case primaryBorderOnly
        case textOnly

        var backgroundColor: Color {
            switch self {
            case .normal:
                NymColor.accent
            case .accentBorderOnly, .textOnly, .primaryBorderOnly:
                .clear
            }
        }

        var imageForegroundColor: Color {
            switch self {
            case .normal:
                NymColor.black
            case .accentBorderOnly, .textOnly:
                NymColor.accent
            case .primaryBorderOnly:
                NymColor.primary
            }
        }

        var textTitleColor: Color {
            switch self {
            case .normal:
                NymColor.black
            case .accentBorderOnly:
                NymColor.accent
            case .textOnly, .primaryBorderOnly:
                NymColor.primary
            }
        }

        var strokeLineWidth: CGFloat {
            switch self {
            case .normal, .textOnly:
                0
            case .accentBorderOnly, .primaryBorderOnly:
                1
            }
        }

        var strokeColor: Color {
            switch self {
            case .normal, .textOnly:
                    .clear
            case .accentBorderOnly:
                NymColor.accent
            case .primaryBorderOnly:
                NymColor.primary
            }
        }
    }

    private let title: String
    private let style: Style
    private let height: CGFloat
    private let isWidthExpanded: Bool
    private let systemImageName: String?
    private let isSystemImageFlipped: Bool

    @State private var isHovered = false
    @Binding private var isLoading: Bool

    public init(
        title: String,
        style: Style = .normal,
        height: CGFloat = 56,
        isLoading: Binding<Bool> = .constant(false),
        isWidthExpanded: Bool = true,
        systemImageName: String? = nil,
        isSystemImageFlipped: Bool = false
    ) {
        self.title = title
        self.style = style
        self.height = height
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
                    .foregroundStyle(style.textTitleColor)
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
            style.backgroundColor.opacity(isHovered ? 0.7 : 1)
        }
        .overlay(
            RoundedRectangle(cornerRadius: 8)
                .stroke(style.strokeColor, lineWidth: style.strokeLineWidth)
        )
        .contentShape(RoundedRectangle(cornerRadius: 8))
        .cornerRadius(8)
        .onHover { newValue in
            isHovered = newValue
        }
    }
}
