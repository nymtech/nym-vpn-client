import SwiftUI

public struct PulsingImageView: View {
    public let systemImageName: String
    public let imageColor: Color

    @State private var scale: CGFloat = 1

    public init(systemImageName: String, imageColor: Color) {
        self.systemImageName = systemImageName
        self.imageColor = imageColor
    }

    public var body: some View {
        GenericImage(systemImageName: systemImageName)
            .foregroundStyle(imageColor)
            .frame(width: 50, height: 50)
            .scaleEffect(scale)
            .onAppear {
                withAnimation(
                    Animation.linear(duration: 0.7)
                        .delay(0.2)
                        .repeatForever(autoreverses: true)
                ) {
                    scale = 1.2
                }
            }
    }
}
