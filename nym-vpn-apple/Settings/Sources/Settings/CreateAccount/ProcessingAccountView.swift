import SwiftUI
import AppSettings
import ConnectionManager
import CredentialsManager
import Routes
import Theme
import UIComponents
#if os(iOS)
import ErrorHandler
#endif

public struct ProcessingAccountView: View {
    @EnvironmentObject private var appSettings: AppSettings
    @EnvironmentObject private var connectionManager: ConnectionManager
    @EnvironmentObject private var credentialsManager: CredentialsManager
    @Binding private var path: NavigationPath
    @State private var didFinishAnimatingText = false
    @State private var didSettleAccount = false
    @State private var didShowFinalMessage = false
    @State private var errorMessage: String?
    @State private var currentStep = 1
    @State private var titlesSessionID = UUID()
    @State private var processingStartedAt: Date?
    @State private var titleBlockHeight: CGFloat = 0

    public var body: some View {
        VStack(alignment: .center, spacing: 0) {
            navbar
            Spacer()
                .frame(height: 24)

            StepView(stepCount: 4, currentStep: $currentStep)
            Spacer()
            dotsAnimationView
            Spacer()
                .frame(height: 16)
            statusTextView
            Spacer()
        }
        .frame(maxWidth: MagicNumbers.moreMaxWidth)
        .padding(16)
        .navigationBarBackButtonHidden(true)
        .background {
            titleMeasurementLayer
        }
        .background {
            Color.Nym.background
                .ignoresSafeArea()
        }
        .task {
            processingStartedAt = Date()
            await prepareAccount()
        }
        .onReceive(credentialsManager.$accountSetupPhase) { phase in
            syncCarouselStep(for: phase)
        }
    }

    public init(path: Binding<NavigationPath>) {
        _path = path
    }
}

private extension ProcessingAccountView {
    var navbar: some View {
        CustomNavBar(useElevationBackground: true)
    }

    var dotsAnimationView: some View {
        WaveDotsView()
    }

    @ViewBuilder
    var statusTextView: some View {
        Group {
            if let errorMessage {
                VStack(alignment: .center, spacing: 16) {
                    Text(errorMessage)
                        .textStyle(.Body.Medium.regular)
                        .multilineTextAlignment(.center)
                    NymButton("retry".localizedString, style: .primary) {
                        resetProcessingState()
                        Task { await prepareAccount() }
                    }
                }
            } else if didShowFinalMessage {
                accountReadyMessage
            } else {
                SwitchingTitlesView(
                    pairs: Self.processingCarouselPairs,
                    stepInterval: SwitchingTitlesView.accountProcessingStepInterval,
                    loopUntilExternalFinish: true,
                    didFinishAnimating: $didFinishAnimatingText,
                    timerDidTick: {
                        currentStep = min(currentStep + 1, 4)
                    }
                )
                .id(titlesSessionID)
            }
        }
        .frame(height: titleBlockHeight > 0 ? titleBlockHeight : nil)
    }

    var titleMeasurementLayer: some View {
        VStack(spacing: 16) {
            ForEach(Array(Self.processingCarouselPairs.enumerated()), id: \.offset) { _, pair in
                titlePairMeasurement(title: pair.0, subtitle: pair.1)
            }
            titlePairMeasurement(
                title: "processingAccount.title5".localizedString,
                subtitle: "processingAccount.subtitle5".localizedString
            )
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

    static var processingCarouselPairs: [(String, String)] {
        (2...4).map { index in
            (
                "processingAccount.title\(index)".localizedString,
                "processingAccount.subtitle\(index)".localizedString
            )
        }
    }

    var accountReadyMessage: some View {
        let title = "processingAccount.title5".localizedString
        let subtitle = "processingAccount.subtitle5".localizedString
        return VStack(alignment: .center, spacing: 16) {
            Text(title)
                .textStyle(.Headline.Medium.regular)
                .multilineTextAlignment(.center)
            Text(subtitle)
                .textStyle(.Body.Medium.regular)
                .foregroundColor(NymColor.gray1)
                .multilineTextAlignment(.center)
        }
    }
}

private extension ProcessingAccountView {
    func prepareAccount() async {
        didSettleAccount = false
        do {
            try await ProcessingAccountCoordinator.prepare(
                credentialsManager: credentialsManager,
                mode: .postPurchase,
                canPrefetchZkNyms: connectionManager.canPrefetchZkNymsFromApp
            )
            settleAccount()
        } catch {
#if os(iOS)
            if case CredentialsManagerError.subscriptionVerifying = error {
                errorMessage = CredentialsManagerError.subscriptionVerifying.localizedTitle
                scheduleSubscriptionVerificationRetry()
                return
            }
            errorMessage = ProcessingAccountErrorMapper.localizedMessage(for: error)
#else
            errorMessage = "generalNymError.somethingWentWrong".localizedString
#endif
        }
    }

    func scheduleSubscriptionVerificationRetry() {
        Task { @MainActor in
            try? await Task.sleep(for: .seconds(2))
            guard OnboardingSession.shared.shouldRetryPostPurchaseVerification() else { return }
            errorMessage = nil
            await prepareAccount()
        }
    }

    func settleAccount() {
        didSettleAccount = true
        Task { await finishCarouselAndAdvance() }
    }

    func finishCarouselAndAdvance() async {
        let stepInterval = SwitchingTitlesView.accountProcessingStepInterval
        if let processingStartedAt {
            let elapsed = Date().timeIntervalSince(processingStartedAt)
            let remaining = max(0, stepInterval - elapsed)
            if remaining > 0 {
                try? await Task.sleep(for: .seconds(remaining))
            }
        }
        didFinishAnimatingText = true
        advanceIfReady()
    }

    func advanceIfReady() {
        guard didFinishAnimatingText, didSettleAccount, !didShowFinalMessage else { return }
        didShowFinalMessage = true
        Task { @MainActor in
            try? await Task.sleep(for: .seconds(2))
            let session = OnboardingSession.shared
            session.advance(to: .credentialsReady)
            session.advance(to: .finished)
            session.markPurchaseFlowDismissed()
            if appSettings.welcomeScreenDidDisplay {
                path = .init()
            } else {
                path = .init([HomeLink.technicalOptIns])
            }
        }
    }

    func resetProcessingState() {
        didFinishAnimatingText = false
        didSettleAccount = false
        didShowFinalMessage = false
        errorMessage = nil
        currentStep = 1
        titlesSessionID = UUID()
        processingStartedAt = Date()
    }

    func syncCarouselStep(for phase: AccountSetupPhase) {
        guard let step = AccountSetupPhase.carouselStep(for: phase, postPurchase: true) else { return }
        if step > currentStep {
            currentStep = step
        }
    }
}
