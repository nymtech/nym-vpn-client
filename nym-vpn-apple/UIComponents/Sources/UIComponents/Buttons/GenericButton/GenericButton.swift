import SwiftUI
import Theme

public struct GenericButton: View {
    private let title: String
    private let borderOnly: Bool
    private let mainColor: Color
    private let height: CGFloat
    private let isWidthExpanded: Bool
    private let systemImageNamge: String?
    private let isSystemImageFlipped: Bool

    @State private var isHovered = false
    @Binding private var isLoading: Bool

    public init(
        title: String,
        borderOnly: Bool = false,
        mainColor: Color = NymColor.accent,
        height: CGFloat = 56,
        isLoading: Binding<Bool> = .constant(false),
        isWidthExpanded: Bool = true,
        systemImageNamge: String? = nil,
        isSystemImageFlipped: Bool = false
    ) {
        self.title = title
        self.borderOnly = borderOnly
        self.mainColor = mainColor
        self.height = height
        self.isWidthExpanded = isWidthExpanded
        self.systemImageNamge = systemImageNamge
        self.isSystemImageFlipped = isSystemImageFlipped
        _isLoading = isLoading
    }

    public var body: some View {
        HStack {
            if isLoading {
                ProgressView()
                    .progressViewStyle(CircularProgressViewStyle(tint: NymColor.black))
            } else {
                if let systemImageNamge {
                    Image(systemName: systemImageNamge)
                        .resizable()
                        .scaledToFit()
                        .frame(width: 24, height: 24)
                        .padding(.horizontal, 8)
                        .scaleEffect(x: isSystemImageFlipped ? -1 : 1, y: 1)
                        .foregroundStyle(borderOnly ? mainColor : NymColor.black)
                }

                Text(title)
                    .foregroundStyle(borderOnly ? mainColor : NymColor.black)
                    .textStyle(.Headline.Small.regular)
            }
        }
        .padding(EdgeInsets(top: 10, leading: 16, bottom: 10, trailing: 16))
        .accessibilityLabel(title)
        .accessibilityAddTraits([.isButton])
        .frame(maxWidth: isWidthExpanded ? .infinity : nil)
        .frame(height: height)
        .background {
            borderOnly ? .clear : mainColor.opacity(isHovered ? 0.7 : 1)
        }
        .overlay(
            RoundedRectangle(cornerRadius: 8)
                .stroke(mainColor, lineWidth: borderOnly ? 1 : 0)
        )
        .contentShape(RoundedRectangle(cornerRadius: 8))
        .cornerRadius(8)
        .onHover { newValue in
            isHovered = newValue
        }
    }
}
