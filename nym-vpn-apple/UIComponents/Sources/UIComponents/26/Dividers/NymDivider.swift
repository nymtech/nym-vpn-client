import SwiftUI
import Theme

public struct NymDivider: View {
    private let color: Color

    public init(color: Color = .Nym.divider) {
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
        NymDivider(color: .Nym.divider)
        NymDivider(color: .Nym.primary)
    }
    .padding(NymSpacing.section)
    .background(Color.Nym.background)
}
#endif
