import SwiftUI
import Lottie

public struct AnimationView: View {
    public let animationName: String

    @Binding var isAnimating: Bool

    public init(animationName: String, isAnimating: Binding<Bool>) {
        self.animationName = animationName
        _isAnimating = isAnimating
    }

    public var body: some View {
        LottieView(animation: .named(animationName, bundle: .module))
            .playing(loopMode: .playOnce)
            .animationDidFinish { completed in
                guard completed else { return }
                Task {
                    try? await Task.sleep(for: .seconds(1))
                    Task { @MainActor in
                        isAnimating = false
                    }
                }
            }
    }
}
