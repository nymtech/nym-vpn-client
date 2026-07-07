import SwiftUI
import Combine
import Theme

public struct SwitchingTitlesView: View {
    private let pairs: [(title: String, subtitle: String)]
    private let timerDidTick: () -> Void
    private let tickInterval: TimeInterval
    private let stepAdvanceDelay: TimeInterval
    private let textTransitionDuration: TimeInterval
    private let initialDwell: TimeInterval
    private let holdOnLastPair: Bool
    private let retainLastPairOnFinish: Bool
    private let finalPairDwell: TimeInterval
    private let onIndexChanged: ((Int) -> Void)?

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
        textTransitionDuration: TimeInterval = 0.35,
        initialDwell: TimeInterval = 0,
        holdOnLastPair: Bool = false,
        retainLastPairOnFinish: Bool = false,
        finalPairDwell: TimeInterval = 0,
        onIndexChanged: ((Int) -> Void)? = nil
    ) {
        self.pairs = pairs.map { (title: $0.0, subtitle: $0.1) }
        _didFinishAnimating = didFinishAnimating
        self.timerDidTick = timerDidTick
        self.tickInterval = tickInterval
        self.stepAdvanceDelay = stepAdvanceDelay
        self.textTransitionDuration = textTransitionDuration
        self.initialDwell = initialDwell
        self.holdOnLastPair = holdOnLastPair
        self.retainLastPairOnFinish = retainLastPairOnFinish
        self.finalPairDwell = finalPairDwell
        self.onIndexChanged = onIndexChanged
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
                .contentTransition(.opacity)
                .animation(textTransition, value: currentIndex)

            if !pairs[currentIndex].subtitle.isEmpty {
                Text(pairs[currentIndex].subtitle)
                    .textStyle(.Body.Medium.regular)
                    .foregroundColor(NymColor.gray1)
                    .multilineTextAlignment(.center)
                    .fixedSize(horizontal: false, vertical: true)
                    .lineLimit(2)
                    .minimumScaleFactor(0.9)
                    .contentTransition(.opacity)
                    .animation(textTransition, value: currentIndex)
            }
        }
        .frame(maxWidth: .infinity, alignment: .center)
        .onAppear {
            syncExternalIndex(currentIndex)
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
    var textTransition: Animation {
        .easeInOut(duration: textTransitionDuration)
    }

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
            if stepAdvanceDelay > 0 {
                withAnimation(textTransition) {
                    currentIndex = nextIndex
                }
                syncExternalIndex(nextIndex)
                scheduleDelayedStepBarTick()
            } else {
                commitSyncedAdvance(to: nextIndex)
            }
            if retainLastPairOnFinish, nextIndex == pairs.count - 1 {
                stopTimer()
                scheduleFinishAfterFinalPairDwell()
            }
        } else if holdOnLastPair {
            return
        } else if retainLastPairOnFinish {
            scheduleFinishAfterFinalPairDwell()
        } else {
            currentIndex = 0
            didFinishAnimating = true
        }
    }

    /// Updates copy and notifies the step bar on the same tick.
    func commitSyncedAdvance(to nextIndex: Int) {
        withAnimation(textTransition) {
            currentIndex = nextIndex
        }
        syncExternalIndex(nextIndex)
    }

    func syncExternalIndex(_ index: Int) {
        if let onIndexChanged {
            onIndexChanged(index)
        } else {
            timerDidTick()
        }
    }

    func scheduleDelayedStepBarTick() {
        stepAdvanceTask = Task { @MainActor in
            try? await Task.sleep(for: .seconds(stepAdvanceDelay))
            guard !Task.isCancelled else { return }
            timerDidTick()
        }
    }

    func scheduleFinishAfterFinalPairDwell() {
        initialDwellTask?.cancel()
        guard finalPairDwell > 0 else {
            didFinishAnimating = true
            return
        }
        initialDwellTask = Task { @MainActor in
            try? await Task.sleep(for: .seconds(finalPairDwell))
            guard !Task.isCancelled else { return }
            didFinishAnimating = true
        }
    }
}
