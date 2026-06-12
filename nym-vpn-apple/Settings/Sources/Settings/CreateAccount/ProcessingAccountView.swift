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
            Color.Nym.background
                .ignoresSafeArea()
        }
        .task {
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
                pairs: [
                    ("processingAccount.title2".localizedString, "processingAccount.subtitle2".localizedString),
                    ("processingAccount.title3".localizedString, "processingAccount.subtitle3".localizedString),
                    ("processingAccount.title4".localizedString, "processingAccount.subtitle4".localizedString),
                    ("processingAccount.title5".localizedString, "processingAccount.subtitle5".localizedString)
                ],
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

    var accountReadyMessage: some View {
        VStack(alignment: .center, spacing: 16) {
            Text("processingAccount.accountReady.title".localizedString)
                .textStyle(.Headline.Medium.regular)
                .multilineTextAlignment(.center)
            Text("processingAccount.accountReady.subtitle".localizedString)
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
            guard OnboardingSession.shared.isWithinPostPurchaseVerificationGracePeriod() else { return }
            errorMessage = nil
            await prepareAccount()
        }
    }

    func settleAccount() {
        didSettleAccount = true
        if !didFinishAnimatingText {
            didFinishAnimatingText = true
        }
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
    }

    func syncCarouselStep(for phase: AccountSetupPhase) {
        guard let step = AccountSetupPhase.carouselStep(for: phase, postPurchase: true) else { return }
        if step > currentStep {
            currentStep = step
        }
    }
}
