import SwiftUI
import Lottie

public struct SplashAnimationView: View {
    private let viewModel: SplashAnimationViewModel

    public init(viewModel: SplashAnimationViewModel) {
        self.viewModel = viewModel
    }

    public var body: some View {
        LottieView(animation: .named(viewModel.animationName, bundle: .module))
            .playing(loopMode: .playOnce)
            .animationDidFinish { _ in
                viewModel.didFinishDisplayingAnimation()
            }
    }
}
