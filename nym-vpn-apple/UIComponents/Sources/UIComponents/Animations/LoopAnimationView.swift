import SwiftUI
import Lottie

public struct LoopAnimationView: View {
    public let animationName: String

    public init(animationName: String) {
        self.animationName = animationName
    }

    public var body: some View {
        LottieView(animation: .named(animationName, bundle: .module))
            .playing(loopMode: .loop)
    }
}
