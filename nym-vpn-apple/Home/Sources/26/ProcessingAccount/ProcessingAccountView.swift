import SwiftUI
import Theme
import UIComponents

struct ProcessingAccountView: View {
    @Bindable var viewModel: ProcessingAccountViewModel
    let minHeight: CGFloat

    @State private var titleBlockHeight: CGFloat = 0

    init(viewModel: ProcessingAccountViewModel, minHeight: CGFloat = 0) {
        self.viewModel = viewModel
        self.minHeight = minHeight
    }

    var body: some View {
        ZStack(alignment: .top) {
            measurementLayer
            content
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
    var content: some View {
        VStack(spacing: 0) {
            stepIndicator
            Spacer(minLength: 0)
            WaveDotsView()
            Spacer(minLength: 0)
            Group {
                if let errorMessage = viewModel.errorMessage {
                    errorState(message: errorMessage)
                } else if viewModel.didShowFinalMessage {
                    welcomeMessage
                } else {
                    switchingTitles
                }
            }
            .frame(height: titleBlockHeight > 0 ? titleBlockHeight : nil)
            Spacer(minLength: 0)
        }
    }

    var measurementLayer: some View {
        ZStack(alignment: .top) {
            ForEach(Array(ProcessingAccountView.pairs(for: viewModel.flow).enumerated()), id: \.offset) { _, pair in
                titlePairMeasurement(title: pair.0, subtitle: pair.1)
            }
            welcomeMessage
                .trackHeight { titleBlockHeight = max(titleBlockHeight, $0) }
        }
        .fixedSize(horizontal: false, vertical: true)
        .hidden()
        .accessibilityHidden(true)
        .allowsHitTesting(false)
    }

    func titlePairMeasurement(title: String, subtitle: String) -> some View {
        VStack(alignment: .center, spacing: 16) {
            Text(title)
                .textStyle(.Headline.Medium.regular)
                .multilineTextAlignment(.center)
            Text(subtitle)
                .textStyle(.Body.Medium.regular)
                .multilineTextAlignment(.center)
        }
        .trackHeight { titleBlockHeight = max(titleBlockHeight, $0) }
    }

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
            pairs: ProcessingAccountView.pairs(for: viewModel.flow),
            stepInterval: ProcessingAccountViewModel.processingStepInterval,
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
        .id(viewModel.titlesSessionID)
    }

    var welcomeMessage: some View {
        Text("purchasePlan.welcomeToTruePrivacy".localizedString)
            .textStyle(.Headline.Medium.regular)
            .foregroundStyle(NymColor.primary)
            .multilineTextAlignment(.center)
    }

    func errorState(message: String) -> some View {
        VStack(alignment: .center, spacing: 16) {
            Text(message)
                .textStyle(.Body.Medium.regular)
                .multilineTextAlignment(.center)
            NymButton("retry".localizedString, style: .primary) {
                viewModel.retry()
            }
        }
    }

    static func pairs(for flow: ProcessingFlow) -> [(String, String)] {
        let prefix: String
        switch flow {
        case .createAccount:
            prefix = "processingAccount.createAccount"
        case .login:
            prefix = "processingAccount.login"
        case .postPurchase:
            prefix = "processingAccount"
        }
        return (2...4).map { index in
            (
                "\(prefix).title\(index)".localizedString,
                "\(prefix).subtitle\(index)".localizedString
            )
        }
    }
}
