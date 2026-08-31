import SwiftUI
import Theme

public struct StepView: View {
    private let perStepDuration: Double = 0.3

    let stepCount: Int
    @Binding var currentStep: Int
    let animateInitialFill: Bool
    let initialFillLeadIn: TimeInterval
    let initialFillStepPause: TimeInterval
    let forwardFillStepPause: TimeInterval

    @State private var displayedStep: Int = 0
    @State private var animationTask: Task<Void, Never>?

    public init(
        stepCount: Int,
        currentStep: Binding<Int>,
        animateInitialFill: Bool = true,
        initialFillLeadIn: TimeInterval = 0.3,
        initialFillStepPause: TimeInterval = 0.3,
        forwardFillStepPause: TimeInterval? = nil
    ) {
        self.stepCount = stepCount
        _currentStep = currentStep
        self.animateInitialFill = animateInitialFill
        self.initialFillLeadIn = initialFillLeadIn
        self.initialFillStepPause = initialFillStepPause
        self.forwardFillStepPause = forwardFillStepPause ?? 0.3
    }

    public var body: some View {
        HStack(spacing: 4) {
            ForEach(0..<max(stepCount, 0), id: \.self) { zeroBasedIndex in
                let index = zeroBasedIndex + 1

                ZStack(alignment: .leading) {
                    RoundedRectangle(cornerRadius: 4)
                        .fill(NymColor.elevation)

                    RoundedRectangle(cornerRadius: 4)
                        .fill(NymColor.accent)
                        .scaleEffect(
                            x: index <= displayedStep ? 1 : 0,
                            y: 1,
                            anchor: .leading
                        )
                }
                .frame(maxWidth: .infinity, minHeight: 4, maxHeight: 4)
            }
        }
        .onAppear {
            if animateInitialFill {
                runInitialFill(to: clamped(currentStep))
            } else {
                displayedStep = clamped(currentStep)
            }
        }
        .onChange(of: currentStep) { oldValue, newValue in
            let old = clamped(oldValue)
            let new = clamped(newValue)

            if new > old {
                runForwardFill(from: displayedStep, to: new)
            } else {
                animationTask?.cancel()
                withAnimation(.linear(duration: perStepDuration)) {
                    displayedStep = new
                }
            }
        }
        .onDisappear {
            animationTask?.cancel()
        }
    }
}

private extension StepView {
    func clamped(_ value: Int) -> Int {
        min(max(value, 0), stepCount)
    }

    func runInitialFill(to target: Int) {
        animationTask?.cancel()
        displayedStep = 0

        animationTask = Task { @MainActor in
            try? await Task.sleep(for: .seconds(initialFillLeadIn))
            guard target > 0 else { return }
            for step in 1...target {
                guard !Task.isCancelled else { return }

                withAnimation(.linear(duration: perStepDuration)) {
                    displayedStep = step
                }
                try? await Task.sleep(for: .seconds(initialFillStepPause))
            }
        }
    }

    func runForwardFill(from current: Int, to target: Int) {
        guard target > current else { return }
        animationTask?.cancel()
        animationTask = Task { @MainActor in
            let start = min(max(displayedStep, current), target)

            for step in (start + 1)...target {
                guard !Task.isCancelled
                else { return }

                withAnimation(.linear(duration: perStepDuration)) {
                    displayedStep = step
                }

                try? await Task.sleep(for: .seconds(forwardFillStepPause))
            }
        }
    }
}
