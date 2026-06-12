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
            content
        }
        .background {
            measurementLayer
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
        let accountReady = Self.accountReadyCopy(for: viewModel.flow)
        return VStack(spacing: 16) {
            ForEach(Array(ProcessingAccountView.pairs(for: viewModel.flow).enumerated()), id: \.offset) { _, pair in
                titlePairMeasurement(title: pair.0, subtitle: pair.1)
            }
            titlePairMeasurement(title: accountReady.title, subtitle: accountReady.subtitle)
        }
        .fixedSize(horizontal: false, vertical: true)
        .opacity(0)
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
            stepCount: viewModel.flow.carouselStepCount,
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
            loopUntilExternalFinish: viewModel.loopsCarouselUntilWorkCompletes,
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
        let accountReady = Self.accountReadyCopy(for: viewModel.flow)
        return VStack(alignment: .center, spacing: 16) {
            Text(accountReady.title)
                .textStyle(.Headline.Medium.regular)
                .foregroundStyle(NymColor.primary)
                .multilineTextAlignment(.center)
            Text(accountReady.subtitle)
                .textStyle(.Body.Medium.regular)
                .foregroundColor(NymColor.gray1)
                .multilineTextAlignment(.center)
        }
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

    static func accountReadyCopy(for flow: ProcessingFlow) -> (title: String, subtitle: String) {
        switch flow {
        case .postPurchase:
            return (
                "processingAccount.title5".localizedString,
                "processingAccount.subtitle5".localizedString
            )
        case .login:
            return (
                "processingAccount.login.title5".localizedString,
                "processingAccount.login.subtitle5".localizedString
            )
        case .createAccount:
            return (
                "processingAccount.createAccount.title5".localizedString,
                "processingAccount.createAccount.subtitle5".localizedString
            )
        }
    }
}
