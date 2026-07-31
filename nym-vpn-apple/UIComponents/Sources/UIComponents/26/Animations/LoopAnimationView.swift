import SwiftUI
import Lottie

public struct LoopAnimationView: View {
    public let animationName: String
    public let fillColor: Color?

    @Environment(\.self) private var environment

    public init(animationName: String, fillColor: Color? = nil) {
        self.animationName = animationName
        self.fillColor = fillColor
    }

    public var body: some View {
        if let fillColor {
            animationView
                .valueProvider(
                    ColorValueProvider(lottieColor(from: fillColor)),
                    for: AnimationKeypath(keypath: "**.Color")
                )
        } else {
            animationView
        }
    }
}

private extension LoopAnimationView {
    var animationView: LottieView<EmptyView> {
        LottieView(animation: .named(animationName, bundle: .module))
            .playing(loopMode: .loop)
            .backgroundBehavior(.pauseAndRestore)
    }

    func lottieColor(from color: Color) -> LottieColor {
        let resolved = color.resolve(in: environment)
        return LottieColor(
            r: Double(resolved.red),
            g: Double(resolved.green),
            b: Double(resolved.blue),
            a: Double(resolved.opacity)
        )
    }
}
