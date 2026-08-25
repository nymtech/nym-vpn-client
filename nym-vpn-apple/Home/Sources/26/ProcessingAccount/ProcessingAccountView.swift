import SwiftUI
import AccountPrefetchGates
import Theme
import UIComponents

struct ProcessingAccountView: View {
    @Bindable var viewModel: ProcessingAccountViewModel
    let minHeight: CGFloat

    @Environment(\.scenePhase) private var scenePhase
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
        .onAppear {
            if scenePhase == .active {
                viewModel.noteCarouselResumed()
            } else {
                viewModel.noteCarouselInterrupted()
            }
        }
        .onDisappear {
            viewModel.noteCarouselInterrupted()
        }
        .onChange(of: scenePhase) { _, newPhase in
            if newPhase == .active {
                viewModel.noteCarouselResumed()
            } else {
                viewModel.noteCarouselInterrupted()
            }
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
            switch ProcessingAccountView.titleBlockMode(
                usesStaticCopy: viewModel.usesStaticCopy,
                didShowFinalMessage: viewModel.didShowFinalMessage,
                showsCredentialsCarousel: showsCredentialsCarousel
            ) {
            case .staticCopy:
                staticTitleView
            case .welcome:
                welcomeMessage
            case .credentials:
                credentialsTitleView
            case .setupCarousel:
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
            ForEach(Array(LoginProcessingUI.setupCarouselPairs().enumerated()), id: \.offset) { _, pair in
                titlePairMeasurement(title: pair.0, subtitle: pair.1)
            }
            ForEach(Array(LoginProcessingUI.credentialsCarouselPairs().enumerated()), id: \.offset) { _, pair in
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
                animateInitialFill: LoginProcessingUI.stepBarAnimateInitialFill,
                initialFillLeadIn: LoginProcessingUI.stepBarInitialLeadIn,
                initialFillStepPause: LoginProcessingUI.stepBarStepPause,
                forwardFillStepPause: LoginProcessingUI.stepBarStepPause
            )
        }
    }

    @ViewBuilder
    var credentialsTitleView: some View {
        if let pair = viewModel.credentialsDisplayPair {
            VStack(alignment: .center, spacing: AuthLayout.processingCarouselTitleSpacing) {
                Text(pair.0)
                    .textStyle(.Headline.Medium.regular)
                    .foregroundStyle(NymColor.primary)
                    .multilineTextAlignment(.center)
                    .contentTransition(.opacity)
                Text(pair.1)
                    .textStyle(.Body.Medium.regular)
                    .foregroundColor(NymColor.gray1)
                    .multilineTextAlignment(.center)
                    .contentTransition(.opacity)
            }
            .animation(
                .easeInOut(duration: LoginProcessingUI.setupCarouselTextTransitionDuration),
                value: pair.0
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
                get: { viewModel.didFinishSetupCarousel },
                set: { newValue in
                    if newValue { viewModel.animationDidFinish() }
                }
            ),
            timerDidTick: {},
            tickInterval: LoginProcessingUI.setupCarouselTickInterval,
            stepAdvanceDelay: LoginProcessingUI.setupCarouselStepAdvanceDelay,
            textTransitionDuration: LoginProcessingUI.setupCarouselTextTransitionDuration,
            initialDwell: LoginProcessingUI.setupCarouselInitialDwell,
            retainLastPairOnFinish: true,
            finalPairDwell: LoginProcessingUI.setupCarouselFinalPairDwell,
            onIndexChanged: { index in
                viewModel.noteSetupCarouselStepBarTick(atIndex: index)
            }
        )
    }

    var showsCredentialsCarousel: Bool {
        LoginProcessingCarouselVisibilityPolicy.showsCredentialsCopy(
            usesStaticCopy: viewModel.usesStaticCopy,
            didShowFinalMessage: viewModel.didShowFinalMessage,
            isSyncing: viewModel.phase == .syncing,
            isPrefetching: viewModel.phase == .prefetching,
            holdsPrefetchCopyThroughAdvance: viewModel.phase == .awaitingAdvance
                && viewModel.hasReachedPrefetchPhase,
            didFinishSetupCarousel: viewModel.didFinishSetupCarousel
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
        case .login, .createAccount:
            return processingCarouselPairs().first ?? ("", "")
        case .postPurchase:
            return (
                PostPurchaseProcessingUI.titleKey.localizedString,
                PostPurchaseProcessingUI.subtitleKey.localizedString
            )
        }
    }

    static func pairs(for flow: ProcessingFlow) -> [(String, String)] {
        switch flow {
        case .login, .createAccount:
            return processingCarouselPairs()
        case .postPurchase:
            return [staticPair(for: .postPurchase)]
        }
    }

    static func processingCarouselPairs() -> [(String, String)] {
        LoginProcessingUI.setupCarouselPairs()
    }
}

enum ProcessingAccountTitleBlockMode: Equatable {
    case staticCopy
    case welcome
    case credentials
    case setupCarousel
}

extension ProcessingAccountView {
    static func titleBlockMode(
        usesStaticCopy: Bool,
        didShowFinalMessage: Bool,
        showsCredentialsCarousel: Bool
    ) -> ProcessingAccountTitleBlockMode {
        if usesStaticCopy {
            return .staticCopy
        }
        if didShowFinalMessage {
            return .welcome
        }
        if showsCredentialsCarousel {
            return .credentials
        }
        return .setupCarousel
    }
}
