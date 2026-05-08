import SwiftUI
import Theme

public struct ExternalLinkImage: View {
    private let color: Color

    public init(color: Color) {
        self.color = color
    }

    public var body: some View {
        Image("externalLink", bundle: .module)
            .resizable()
            .frame(width: 16, height: 16)
            .foregroundColor(color)
    }
}
