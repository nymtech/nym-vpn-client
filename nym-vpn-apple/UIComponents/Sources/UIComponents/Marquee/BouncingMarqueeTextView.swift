import SwiftUI
import Theme

public struct BouncingMarqueeTextView: View {
    let text: String
    let textStyle: NymTextStyle
    let fontColor: Color
    let speed: Double // Speed in points per second
    let pauseDuration: Double // Pause duration at the start and end

    @State private var textWidth: CGFloat = 0
    @State private var containerWidth: CGFloat = 0
    @State private var offset: CGFloat = 0
    @State private var isReversing = false

    public var body: some View {
        GeometryReader { geo in
            HStack {
                Text(text)
                    .foregroundStyle(fontColor)
                    .textStyle(textStyle)
                    .fixedSize()
                    .background(
                        GeometryReader { textGeo in
                            Color.clear
                                .onAppear {
                                    DispatchQueue.main.asyncAfter(deadline: .now() + 1.0) {
                                        textWidth = textGeo.size.width
                                        containerWidth = geo.size.width
                                        startAnimationIfNeeded()
                                    }
                                }
                        }
                    )
                    .offset(x: offset)
            }
            .clipped()
        }
        .onChange(of: text) { _ in
            resetAnimationIfNeeded()
        }
    }
}

private extension BouncingMarqueeTextView {
    func resetAnimationIfNeeded() {
        textWidth = 0
        offset = 0
        isReversing = false
        startAnimationIfNeeded()
    }

    func startAnimationIfNeeded() {
        guard textWidth > containerWidth
        else {
            offset = 0
            return
        }

        startAnimation()
    }

    func startAnimation() {
        Task {
            let maxOffset = containerWidth - textWidth
            let targetOffset = isReversing ? 0 : maxOffset
            let distance = abs(offset - targetOffset)
            let duration = distance / speed

            withAnimation(.linear(duration: duration)) {
                offset = targetOffset
            }

            DispatchQueue.main.asyncAfter(deadline: .now() + duration + pauseDuration) {
                isReversing.toggle()
                startAnimationIfNeeded()
            }
        }
    }
}
