import SwiftUI
import Theme

public struct ElevationSectionView<Content: View>: View {
    private let content: Content

    public var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            content
        }
        .padding(16)
        .frame(maxWidth: .infinity, alignment: .topLeading)
        .background(Color.Nym.surface)
        .cornerRadius(8)
    }

    public init(@ViewBuilder content: () -> Content) {
        self.content = content()
    }
}
