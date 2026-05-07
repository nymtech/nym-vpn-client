import SwiftUI
import Theme
import UIComponents

struct ProcessingAccountView: View {
    @Bindable var viewModel: ProcessingAccountViewModel
    let minHeight: CGFloat

    init(viewModel: ProcessingAccountViewModel, minHeight: CGFloat = 0) {
        self.viewModel = viewModel
        self.minHeight = minHeight
    }

    var body: some View {
        VStack(spacing: AuthLayout.stackSpacing) {
            stepIndicator
            Spacer(minLength: 0)
            WaveDotsView()
            Spacer().frame(height: NymSpacing.large)
            if viewModel.didShowFinalMessage {
                welcomeMessage
            } else {
                switchingTitles
            }
            Spacer(minLength: 0)
        }
        .padding(.horizontal, NymSpacing.component)
        .padding(.vertical, AuthLayout.verticalPadding)
        .frame(maxWidth: .infinity)
        .frame(height: minHeight > 0 ? minHeight : nil)
        .task {
            viewModel.start()
        }
    }
}

private extension ProcessingAccountView {
    var stepIndicator: some View {
        StepView(
            stepCount: 4,
            currentStep: Binding(
                get: { viewModel.currentStep },
                set: { _ in }
            )
        )
    }

    var switchingTitles: some View {
        SwitchingTitlesView(
            pairs: [
                (
                    "processingAccount.title2".localizedString,
                    "processingAccount.subtitle2".localizedString
                ),
                (
                    "processingAccount.title3".localizedString,
                    "processingAccount.subtitle3".localizedString
                ),
                (
                    "processingAccount.title4".localizedString,
                    "processingAccount.subtitle4".localizedString
                )
            ],
            didFinishAnimating: Binding(
                get: { viewModel.didFinishAnimatingText },
                set: { newValue in
                    if newValue { viewModel.animationDidFinish() }
                }
            ),
            timerDidTick: {
                viewModel.animationDidAdvance()
            }
        )
    }

    var welcomeMessage: some View {
        Text("purchasePlan.welcomeToTruePrivacy".localizedString)
            .textStyle(.Headline.Medium.regular)
            .foregroundStyle(NymColor.primary)
            .multilineTextAlignment(.center)
    }
}
