import SwiftUI
import Theme

public struct NymDivider: View {
    private let color: Color

    public init(color: Color = .Nym.surfaceHair) {
        self.color = color
    }

    public var body: some View {
        Rectangle()
            .fill(color)
            .frame(height: 1)
            .accessibilityHidden(true)
    }
}

#if DEBUG
#Preview {
    VStack(spacing: NymSpacing.medium) {
        NymDivider()
        NymDivider(color: .Nym.white6)
        NymDivider(color: .Nym.brandPrimary)
    }
    .padding(NymSpacing.section)
    .background(Color.Nym.surfaceBg)
}
#endif
