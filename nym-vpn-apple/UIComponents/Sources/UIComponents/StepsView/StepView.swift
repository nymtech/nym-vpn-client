import SwiftUI
import Theme

public struct StepView: View {
    let stepCount: Int
    let currentStep: Int

    @State private var animateFill = false
    private let perStepDuration: Double = 0.3

    public init(stepCount: Int, currentStep: Int) {
        self.stepCount = stepCount
        self.currentStep = currentStep
    }

    public var body: some View {
        HStack(spacing: 4) {
            ForEach(1...stepCount, id: \.self) { index in
                ZStack(alignment: .leading) {
                    RoundedRectangle(cornerRadius: 4)
                        .fill(NymColor.elevation)

                    RoundedRectangle(cornerRadius: 4)
                        .fill(NymColor.accent)
                        .scaleEffect(
                            x: (animateFill && index <= currentStep) ? 1 : 0,
                            y: 1,
                            anchor: .leading
                        )
                        .animation(
                            .linear(duration: perStepDuration)
                            .delay(perStepDuration * Double(index - 1)),
                            value: animateFill
                        )
                }
                .frame(maxWidth: .infinity, minHeight: 4, maxHeight: 4)
            }
        }
        .onAppear {
            animateFill = false
            DispatchQueue.main.asyncAfter(
                deadline: .now() + 0.3,
                execute: {
                    animateFill = true
                }
            )
        }
        .onChange(of: currentStep) { _, _ in
            animateFill = false
            DispatchQueue.main.asyncAfter(
                deadline: .now() + 0.3,
                execute: {
                    animateFill = true
                }
            )
        }
    }
}
