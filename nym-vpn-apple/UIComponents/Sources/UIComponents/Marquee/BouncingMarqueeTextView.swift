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
        Text(" ")
            .textStyle(textStyle)
            .lineLimit(1)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(
                GeometryReader { geometry in
                    Color.clear
                        .onAppear {
                            containerWidth = geometry.size.width
                            startAnimationIfNeeded()
                        }
                        .onChange(of: geometry.size.width) { width in
                            containerWidth = width
                            resetAnimationIfNeeded()
                        }
                }
            )
        // Overlay holds the moving text; overlays don't affect layout size.
            .overlay(alignment: .leading) {
                ZStack(alignment: .leading) {
                    // Visible text (never truncates because it renders at intrinsic width)
                    Text(text)
                        .foregroundStyle(fontColor)
                        .textStyle(textStyle)
                        .lineLimit(1)
                        .fixedSize(horizontal: true, vertical: false)
                        .offset(x: offset)

                    Text(text)
                        .textStyle(textStyle)
                        .lineLimit(1)
                        .fixedSize(horizontal: true, vertical: false)
                        .opacity(0.0001)
                        .accessibilityHidden(true)
                        .background(
                            GeometryReader { geometry in
                                Color.clear
                                    .onAppear {
                                        textWidth = geometry.size.width
                                        startAnimationIfNeeded()
                                    }
                                    .onChange(of: geometry.size.width) { width in
                                        textWidth = width
                                        resetAnimationIfNeeded()
                                    }
                            }
                        )
                }
                .frame(width: containerWidth, alignment: .leading)
                .clipped()
            }
            .onChange(of: text) { _ in
                resetAnimationIfNeeded()
            }
    }
}

private extension BouncingMarqueeTextView {
    func resetAnimationIfNeeded() {
        isReversing = false
        offset = 0
        startAnimationIfNeeded()
    }

    func startAnimationIfNeeded() {
        guard textWidth > 0, containerWidth > 0 else { return }
        guard textWidth > containerWidth
        else {
            withAnimation { offset = 0 }
            return
        }
        startAnimation()
    }

    func startAnimation() {
        let maxOffset = containerWidth - textWidth
        let target = isReversing ? 0 : maxOffset
        let distance = abs(offset - target)
        let duration = distance / max(speed, 1)

        withAnimation(.linear(duration: duration)) {
            offset = target
        }

        DispatchQueue.main.asyncAfter(deadline: .now() + duration + pauseDuration) {
            isReversing.toggle()
            startAnimationIfNeeded()
        }
    }
}
