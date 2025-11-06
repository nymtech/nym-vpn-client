import SwiftUI
import Theme

public struct WaveDotsView: View {
    private static let animationInterval: TimeInterval = 0.3
    private let timer = Timer
        .publish(every: WaveDotsView.animationInterval, on: .main, in: .common)
        .autoconnect()

    private let dotCount = 4
    private let maxDotScale = 1.0
    private let minDotScale = 0.5
    private let dotDiameter = CGFloat(8)
    private let squareSide: CGFloat = 68
    private let cornerRadius: CGFloat = 8

    @State private var currentIndex: Int = -1

    public var body: some View {
        ZStack {
            RoundedRectangle(cornerRadius: cornerRadius)
                .fill(Color(red: 0.07, green: 0.77, blue: 0.37).opacity(0.15))
                .frame(width: squareSide, height: squareSide)
                .overlay(
                    RoundedRectangle(cornerRadius: cornerRadius)
                        .stroke(Color(red: 0.08, green: 0.91, blue: 0.44).opacity(0.25), lineWidth: 1)
                )

            HStack(spacing: 2) {
                ForEach(0..<dotCount, id: \.self) { index in
                    Circle()
                        .frame(width: dotDiameter, height: dotDiameter)
                        .scaleEffect(index == currentIndex ? maxDotScale : minDotScale)
                        .animation(
                            .easeInOut(
                                duration: WaveDotsView.animationInterval
                            ),
                            value: currentIndex
                        )
                        .foregroundColor(NymColor.action)
                }
            }
            .padding(12)
        }
        .onReceive(timer) { _ in
            currentIndex = (currentIndex + 1) % dotCount
        }
    }

    public init() {}
}
