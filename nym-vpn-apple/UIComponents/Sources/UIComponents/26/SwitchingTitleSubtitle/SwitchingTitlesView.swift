import SwiftUI
import Combine
import Theme

public struct SwitchingTitlesView: View {
    private let pairs: [(title: String, subtitle: String)]
    private let timerDidTick: () -> Void
    private let tickInterval: TimeInterval
    private let stepAdvanceDelay: TimeInterval
    private let initialDwell: TimeInterval
    private let holdOnLastPair: Bool

    @State private var currentIndex = 0
    @State private var timerCancellable: AnyCancellable?
    @State private var stepAdvanceTask: Task<Void, Never>?
    @State private var initialDwellTask: Task<Void, Never>?

    @Binding var didFinishAnimating: Bool

    public init(
        pairs: [(String, String)],
        didFinishAnimating: Binding<Bool>,
        timerDidTick: @escaping () -> Void,
        tickInterval: TimeInterval = 2.0,
        stepAdvanceDelay: TimeInterval = 0,
        initialDwell: TimeInterval = 0,
        holdOnLastPair: Bool = false
    ) {
        self.pairs = pairs.map { (title: $0.0, subtitle: $0.1) }
        _didFinishAnimating = didFinishAnimating
        self.timerDidTick = timerDidTick
        self.tickInterval = tickInterval
        self.stepAdvanceDelay = stepAdvanceDelay
        self.initialDwell = initialDwell
        self.holdOnLastPair = holdOnLastPair
    }

    public var body: some View {
        VStack(alignment: .center, spacing: 12) {
            Text(pairs[currentIndex].title)
                .textStyle(.Headline.Medium.regular)
                .foregroundStyle(NymColor.primary)
                .multilineTextAlignment(.center)
                .fixedSize(horizontal: false, vertical: true)
                .lineLimit(2)
                .minimumScaleFactor(0.9)

            if !pairs[currentIndex].subtitle.isEmpty {
                Text(pairs[currentIndex].subtitle)
                    .textStyle(.Body.Medium.regular)
                    .foregroundColor(NymColor.gray1)
                    .multilineTextAlignment(.center)
                    .fixedSize(horizontal: false, vertical: true)
                    .lineLimit(2)
                    .minimumScaleFactor(0.9)
            }
        }
        .frame(maxWidth: .infinity, alignment: .center)
        .onAppear {
            beginCarousel()
        }
        .onDisappear {
            stopTimer()
            stepAdvanceTask?.cancel()
            initialDwellTask?.cancel()
        }
    }
}

private extension SwitchingTitlesView {
    func beginCarousel() {
        guard initialDwell > 0 else {
            startTimer()
            return
        }
        initialDwellTask = Task { @MainActor in
            try? await Task.sleep(for: .seconds(initialDwell))
            guard !Task.isCancelled else { return }
            startTimer()
        }
    }

    func startTimer() {
        stopTimer()

        timerCancellable = Timer.publish(every: tickInterval, on: .main, in: .common)
            .autoconnect()
            .sink { _ in
                advanceIndex()
            }
    }

    func stopTimer() {
        timerCancellable?.cancel()
        timerCancellable = nil
    }

    func advanceIndex() {
        stepAdvanceTask?.cancel()
        let nextIndex = currentIndex + 1
        if nextIndex < pairs.count {
            currentIndex = nextIndex
            scheduleStepAdvance()
        } else if holdOnLastPair {
            return
        } else {
            currentIndex = 0
            didFinishAnimating = true
        }
    }

    func scheduleStepAdvance() {
        guard stepAdvanceDelay > 0 else {
            timerDidTick()
            return
        }
        stepAdvanceTask = Task { @MainActor in
            try? await Task.sleep(for: .seconds(stepAdvanceDelay))
            guard !Task.isCancelled else { return }
            timerDidTick()
        }
    }
}
