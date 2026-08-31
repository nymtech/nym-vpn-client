import SwiftUI
import AccountPrefetchGates
import CredentialsManager
import Theme
import UIComponents

public struct ProcessingAccountView: View {
    @EnvironmentObject private var credentialsManager: CredentialsManager
    @Binding private var path: NavigationPath
    private let onPurchaseFlowComplete: (() -> Void)?
    private let onPurchaseFlowDismissed: (() -> Void)?

    @State private var didCompleteAccountPrep = false
    @State private var didNavigate = false

    public var body: some View {
        VStack(alignment: .center, spacing: 0) {
            navbar
            Spacer()
                .frame(height: 24)

            if ProcessingUIPolicy.showsOnboardingProgressBar(usesStaticCopy: true) {
                StepView(
                    stepCount: 4,
                    currentStep: .constant(PostPurchaseProcessingUI.progressStep),
                    animateInitialFill: false
                )
            }
            Spacer()
            dotsAnimationView
            Spacer()
                .frame(height: 16)
            staticTextView
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
            let credentials = credentialsManager
            let outcome = await AccountPrefetchOrchestrator.runPostPurchaseProcessingFlow(
                syncSubscriptionPayment: {
                    try await credentials.handleSubscriptionPayment()
                },
                isAccountActive: {
                    await credentials.isAccountActive()
                },
                prefetchZkNyms: {
                    await credentials.prefetchZkNyms()
                }
            )
            guard !Task.isCancelled else { return }
            guard PostPurchaseProcessingPolicy.shouldCompleteNavigation(
                didSyncSubscription: outcome.didSyncSummary,
                isAccountActive: outcome.isAccountActive
            ) else {
                dismissProcessing()
                return
            }
            didCompleteAccountPrep = true
            advanceIfReady()
        }
    }

    public init(
        path: Binding<NavigationPath>,
        onPurchaseFlowComplete: (() -> Void)? = nil,
        onPurchaseFlowDismissed: (() -> Void)? = nil
    ) {
        _path = path
        self.onPurchaseFlowComplete = onPurchaseFlowComplete
        self.onPurchaseFlowDismissed = onPurchaseFlowDismissed
    }
}

private extension ProcessingAccountView {
    var navbar: some View {
        CustomNavBar(
            useElevationBackground: true,
            rightButton: CustomNavBarButton(
                type: .close,
                action: { dismissProcessing() }
            )
        )
    }

    var dotsAnimationView: some View {
        WaveDotsView()
    }

    var staticTextView: some View {
        VStack(alignment: .center, spacing: 16) {
            Text(PostPurchaseProcessingUI.titleKey.localizedString)
                .textStyle(.Headline.Medium.regular)
                .foregroundStyle(NymColor.primary)
                .multilineTextAlignment(.center)

            Text(PostPurchaseProcessingUI.subtitleKey.localizedString)
                .textStyle(.Body.Medium.regular)
                .foregroundColor(NymColor.gray1)
                .multilineTextAlignment(.center)
        }
    }

    func advanceIfReady() {
        guard !didNavigate,
              ProcessingAccountReadiness.canAdvanceNavigation(
                  didCompleteAccountPrep: didCompleteAccountPrep,
                  didFinishAnimatingText: true,
                  requiresCarousel: false
              ) else { return }
        didNavigate = true
        navigateHome()
    }

    func navigateHome() {
        onPurchaseFlowComplete?()
        path = .init()
    }

    func dismissProcessing() {
        onPurchaseFlowDismissed?()
        path = .init()
    }
}
