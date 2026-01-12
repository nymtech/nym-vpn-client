import SwiftUI
import Combine
import Theme

public struct SwitchingTitlesView: View {
    private let pairs: [(title: String, subtitle: String)]
    private let timerDidTick: () -> Void

    @State private var currentIndex = 0
    @State private var timerCancellable: AnyCancellable?

    @Binding var didFinishAnimating: Bool

    public init(pairs: [(String, String)], didFinishAnimating: Binding<Bool>, timerDidTick: @escaping () -> Void) {
        self.pairs = pairs.map { (title: $0.0, subtitle: $0.1) }
        _didFinishAnimating = didFinishAnimating
        self.timerDidTick = timerDidTick
    }

    public var body: some View {
        VStack(alignment: .center, spacing: 16) {
            Text(pairs[currentIndex].title)
                .textStyle(.Headline.Medium.regular)
                .foregroundStyle(NymColor.primary)
                .multilineTextAlignment(.center)

            Text(pairs[currentIndex].subtitle)
                .textStyle(.Body.Medium.regular)
                .foregroundColor(NymColor.gray1)
                .multilineTextAlignment(.center)
        }
        .onAppear {
            startTimer()
        }
        .onDisappear {
            stopTimer()
        }
    }
}

private extension SwitchingTitlesView {
    func startTimer() {
        stopTimer()

        timerCancellable = Timer.publish(every: 2.0, on: .main, in: .common)
            .autoconnect()
            .sink { _ in
                advanceIndex()
                timerDidTick()
            }
    }

    func stopTimer() {
        timerCancellable?.cancel()
        timerCancellable = nil
    }

    func advanceIndex() {
        let nextIndex = currentIndex + 1
        if nextIndex < pairs.count {
            currentIndex = nextIndex
        } else {
            currentIndex = 0
            didFinishAnimating = true
        }
    }
}
