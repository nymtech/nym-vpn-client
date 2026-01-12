import SwiftUI
import Theme

public struct StepView: View {
    private let perStepDuration: Double = 0.3

    let stepCount: Int
    @Binding var currentStep: Int

    @State private var displayedStep: Int = 0
    @State private var animationID: Int = 0

    public init(stepCount: Int, currentStep: Binding<Int>) {
        self.stepCount = stepCount
        _currentStep = currentStep
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
                        .animation(.linear(duration: perStepDuration), value: displayedStep)
                }
                .frame(maxWidth: .infinity, minHeight: 4, maxHeight: 4)
            }
        }
        .onAppear {
            displayedStep = 0
            animateForwardIfNeeded(from: 0, to: clamped(currentStep))
        }
        .onChange(of: currentStep) { oldValue, newValue in
            let old = clamped(oldValue)
            let new = clamped(newValue)

            if new > old {
                animateForwardIfNeeded(from: old, to: new)
            } else {
                animationID += 1
                withAnimation(.linear(duration: perStepDuration)) {
                    displayedStep = new
                }
            }
        }
    }
}

private extension StepView {
    func clamped(_ value: Int) -> Int {
        min(max(value, 0), stepCount)
    }

    func animateForwardIfNeeded(from old: Int, to new: Int) {
        guard new > old
        else { return }

        animationID += 1
        let id = animationID

        let start = min(max(displayedStep, old), new)

        (start + 1...new).forEach { next in
            let delay = perStepDuration * Double(next - start - 1)

            DispatchQueue.main.asyncAfter(deadline: .now() + delay) {
                guard id == animationID
                else { return }

                withAnimation(.linear(duration: perStepDuration)) {
                    displayedStep = next
                }
            }
        }
    }
}
