import SwiftUI
import AccountPrefetchGates
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
            if viewModel.flow == .postPurchase {
                dismissControl
            }
        }
        .padding(.horizontal, NymSpacing.component)
        .padding(.vertical, AuthLayout.processingCarouselVerticalPadding)
        .frame(maxWidth: .infinity)
        .frame(height: minHeight > 0 ? minHeight : nil, alignment: .top)
        .task {
            viewModel.start()
        }
    }
}

private extension ProcessingAccountView {
    var dismissControl: some View {
        HStack {
            Spacer()
            Button {
                viewModel.dismissPostPurchaseProcessing()
            } label: {
                Image(systemName: "xmark")
                    .font(.body.weight(.semibold))
                    .foregroundStyle(NymColor.gray1)
                    .frame(width: 44, height: 44)
            }
            .accessibilityLabel("cancel".localizedString)
        }
    }
}

private extension ProcessingAccountView {
    var content: some View {
        VStack(spacing: AuthLayout.processingCarouselStackSpacing) {
            AuthDrawerHeader(showsBackButton: false)
            stepIndicator
            WaveDotsView()
                .padding(.top, AuthLayout.carouselLoaderTopSpacing)
                .padding(.bottom, AuthLayout.carouselLoaderBottomSpacing)
            titleBlock
        }
        .frame(maxWidth: .infinity, alignment: .top)
    }

    @ViewBuilder
    var titleBlock: some View {
        Group {
            if viewModel.usesStaticCopy {
                staticTitleView
            } else if viewModel.didShowFinalMessage {
                welcomeMessage
            } else {
                switchingTitles
            }
        }
        .frame(
            height: AuthLayout.processingCarouselTitleReservedHeight(
                didShowFinalMessage: viewModel.didShowFinalMessage,
                measuredCarouselTitleHeight: titleBlockHeight
            )
        )
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
        VStack(alignment: .center, spacing: AuthLayout.processingCarouselTitleSpacing) {
            Text(title)
                .textStyle(.Headline.Medium.regular)
                .multilineTextAlignment(.center)
            Text(subtitle)
                .textStyle(.Body.Medium.regular)
                .multilineTextAlignment(.center)
        }
        .trackHeight { titleBlockHeight = max(titleBlockHeight, $0) }
    }

    @ViewBuilder
    var stepIndicator: some View {
        if ProcessingUIPolicy.showsOnboardingProgressBar(usesStaticCopy: viewModel.usesStaticCopy) {
            StepView(
                stepCount: 4,
                currentStep: Binding(
                    get: { viewModel.currentStep },
                    set: { _ in }
                ),
                animateInitialFill: !viewModel.usesStaticCopy
            )
        }
    }

    var staticTitleView: some View {
        let pair = ProcessingAccountView.staticPair(for: viewModel.flow)
        return VStack(alignment: .center, spacing: AuthLayout.processingCarouselTitleSpacing) {
            Text(pair.0)
                .textStyle(.Headline.Medium.regular)
                .foregroundStyle(NymColor.primary)
                .multilineTextAlignment(.center)
            Text(pair.1)
                .textStyle(.Body.Medium.regular)
                .foregroundColor(NymColor.gray1)
                .multilineTextAlignment(.center)
        }
    }

    var switchingTitles: some View {
        SwitchingTitlesView(
            pairs: ProcessingAccountView.pairs(for: viewModel.flow),
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

    static func staticPair(for flow: ProcessingFlow) -> (String, String) {
        switch flow {
        case .login:
            return loginCarouselPairs().first ?? ("", "")
        case .postPurchase:
            return (
                PostPurchaseProcessingUI.titleKey.localizedString,
                PostPurchaseProcessingUI.subtitleKey.localizedString
            )
        case .createAccount:
            return ("", "")
        }
    }

    static func pairs(for flow: ProcessingFlow) -> [(String, String)] {
        switch flow {
        case .login:
            return loginCarouselPairs()
        case .postPurchase:
            return [staticPair(for: .postPurchase)]
        case .createAccount:
            let prefix = "processingAccount.createAccount"
            return (2...4).map { index in
                (
                    "\(prefix).title\(index)".localizedString,
                    "\(prefix).subtitle\(index)".localizedString
                )
            }
        }
    }

    static func loginCarouselPairs() -> [(String, String)] {
        let title = LoginProcessingUI.settingUpTitleKey.localizedString
        return LoginProcessingUI.carouselStepRange.map { _ in (title, "") }
    }
}
