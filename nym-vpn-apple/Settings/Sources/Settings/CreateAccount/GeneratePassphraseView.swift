import SwiftUI
import StoreKit
import AppSettings
import AccountPrefetchGates
import CredentialsManager
import ExternalLinkManager
#if os(iOS)
import ImpactGenerator
import NymVPNLib
import ErrorHandler
import PurchasesManager
#endif
import Theme
import UIComponents

public struct GeneratePassphraseView: View {
    @Binding private var path: NavigationPath
    @State private var didFinishAnimatingText: Bool
    @State private var currentStep: Int

#if os(iOS)
    @EnvironmentObject var purchasesManager: PurchasesManager
#endif
    @EnvironmentObject var externalLinkManager: ExternalLinkManager

    @State var alertTitle: String = ""
    @State var didRegisterAccount = false
    @State var isRegistering = false
    @State var isAlertDisplayed = false
    @State var isPurchasing = false
    @State var isPlanAlertDisplayed = false
    @State var alertOffersRegistrationRetry = false
    @State private var didLeaveForSuccessfulPurchase = false
#if os(iOS)
    @State private var autologinState = AutologinState()
#elseif os(macOS)
    @State var autologinState = AutologinState()
#endif

    private let isPurchaseOnly: Bool
    private let onPurchaseFlowDismissed: (() -> Void)?

    @EnvironmentObject var appSettings: AppSettings
    @EnvironmentObject var credentialsManager: CredentialsManager

    public var body: some View {
        VStack(spacing: 0) {
            navbar()
            Spacer()
                .frame(height: 24)

            VStack(spacing: 0) {
                if showsOnboardingProgressBar {
                    StepView(
                        stepCount: 4,
                        currentStep: $currentStep,
                        animateInitialFill: !isPurchaseOnly
                    )
                }
                Spacer()

                if !(didFinishAnimatingText && didRegisterAccount) {
                    // Generate account
                    dotsAnimationView
                    Spacer()
                        .frame(height: 16)

                    animatingTextView
                    Spacer()
                } else {
                    // Purchase plan
                    checkmarkImage
                    Spacer()
                        .frame(height: 12)
                    titleSubtitleView
                    Spacer()
#if os(iOS)
                    if showsWebSubscribeOnSubscriptionPage {
                        webSubscribeButton
                        Spacer()
                            .frame(height: 12)
                    }
#endif
                    selectPlanButton
                }
            }
            .frame(maxWidth: MagicNumbers.moreMaxWidth)
        }
        .padding(16)
        .navigationBarBackButtonHidden(true)
        .background {
            Color.Nym.background
                .ignoresSafeArea()
        }
        .task {
            if isPurchaseOnly {
                didRegisterAccount = true
            } else {
                await generateAndRegisterMnemonic()
            }
        }
        .alert(alertTitle, isPresented: $isAlertDisplayed) {
            if alertOffersRegistrationRetry {
                Button("retry".localizedString, role: .cancel) {
                    Task {
                        await generateAndRegisterMnemonic()
                    }
                }
            } else {
                Button("ok".localizedString, role: .cancel) {}
            }
        }
#if os(iOS)
        .autologinOverlay(state: autologinState, onRetry: { beginWebSubscriptionPurchase() })
        .onChange(of: credentialsManager.didReceiveSubscriptionPayment) { _, received in
            guard received else { return }
            autologinState.dismissAfterWebReturn()
        }
#elseif os(macOS)
        .autologinOverlay(state: autologinState)
#endif
        .onDisappear {
            isAlertDisplayed = false
            isPlanAlertDisplayed = false
            guard !didLeaveForSuccessfulPurchase else { return }
            onPurchaseFlowDismissed?()
        }
    }

    public init(
        path: Binding<NavigationPath>,
        displayPurchaseView: Bool = false,
        onPurchaseFlowDismissed: (() -> Void)? = nil
    ) {
        _path = path
        isPurchaseOnly = displayPurchaseView
        self.onPurchaseFlowDismissed = onPurchaseFlowDismissed
        didFinishAnimatingText = displayPurchaseView
        currentStep = displayPurchaseView
            ? OnboardingSessionPolicy.progressStep(for: .iapPurchaseRequired)
            : 1
    }
}

// MARK: - Views -
private extension GeneratePassphraseView {
    var showsOnboardingProgressBar: Bool {
        PurchasePresentationPolicy.showsOnboardingProgressBar(
            isPurchaseOnly: isPurchaseOnly,
            didFinishAnimatingText: didFinishAnimatingText,
            didRegisterAccount: didRegisterAccount
        )
    }

    var showsWebSubscribeOnSubscriptionPage: Bool {
#if os(iOS)
        WebPurchasePresentationPolicy.showsWebOnSubscriptionPage(isIOS: true)
#else
        WebPurchasePresentationPolicy.showsWebOnSubscriptionPage(isIOS: false)
#endif
    }

    @ViewBuilder
    func navbar() -> some View {
        if didFinishAnimatingText {
            CustomNavBar(
                useElevationBackground: true,
                rightButton: CustomNavBarButton(
                    type: .close,
                    action: {
                        path = .init()
                    }
                )
            )
        } else {
            CustomNavBar(useElevationBackground: true)
        }
    }

    var dotsAnimationView: some View {
        WaveDotsView()
    }

    // MARK: - Generate account -
    var animatingTextView: some View {
        SwitchingTitlesView(
            pairs: [
                ("generatePassphrase.title1".localizedString, "generatePassphrase.subtitle1".localizedString),
                ("generatePassphrase.title2".localizedString, "generatePassphrase.subtitle2".localizedString),
                ("generatePassphrase.title3".localizedString, "generatePassphrase.subtitle3".localizedString)
            ],
            didFinishAnimating: $didFinishAnimatingText,
            timerDidTick: {
                currentStep += 1
            }
        )
    }

    // MARK: - Plan purchase -
    var checkmarkImage: some View {
        HStack {
            Spacer()
            ZStack {
                RoundedRectangle(cornerRadius: 12)
                    .fill(Color.Nym.primary.opacity(0.15))
                    .frame(width: 68, height: 68)
                    .overlay(
                        RoundedRectangle(cornerRadius: 12)
                            .stroke(Color.Nym.primary.opacity(0.25), lineWidth: 1)
                    )

                GenericImage(imageName: "checkmarkCircle")
                    .frame(width: 46, height: 46)
            }
            Spacer()
        }
    }

    var titleSubtitleView: some View {
        VStack(alignment: .center) {
            Text("purchasePlan.title".localizedString)
                .nymTextStyle(.titleScreen)
                .foregroundStyle(Color.Nym.textPrimary)
                .multilineTextAlignment(.center)

            Spacer()
                .frame(height: 24)
            HStack {
                Spacer()
                subsctiptionBenefitsSection
                Spacer()
            }
        }.padding(.horizontal, 32)
    }

    var subsctiptionBenefitsSection: some View {
        VStack(spacing: 16) {
#if os(iOS)
            if !purchasesManager.isEligibleForIntroOffer.isEmpty {
                subscriptionBenefitCell(
                    imageName: nil,
                    title: "purchasePlan.7dayFreeTrial".localizedString,
                    systemImageName: "gift"
                )
            }
#endif
            subscriptionBenefitCell(imageName: "allFeaturesIncluded", title: "purchasePlan.allFeatures".localizedString)
            subscriptionBenefitCell(imageName: "noAds", title: "purchasePlan.noAds".localizedString)
            subscriptionBenefitCell(imageName: "cancelAnytime", title: "purchasePlan.cancelAnytime".localizedString)
        }
    }

    func subscriptionBenefitCell(imageName: String?, title: String, systemImageName: String? = nil) -> some View {
        HStack(spacing: 8) {
            if let systemImageName {
                GenericImage(systemImageName: systemImageName)
                    .foregroundStyle(Color.Nym.primary)
                    .frame(width: 24, height: 24)
            } else if let imageName {
                GenericImage(imageName: imageName)
                    .foregroundStyle(Color.Nym.primary)
                    .frame(width: 24, height: 24)
            }
            Text(title)
                .foregroundStyle(Color.Nym.textSecondary)
                .nymTextStyle(.bodyDefault)
            Spacer()
        }
        .accessibilityElement(children: .combine)
        .accessibilityLabel(title)
    }

    var selectPlanButton: some View {
        GenericButton(title: "purchasePlan.selectPlan".localizedString, isLoading: $isPurchasing)
            .onTapGesture {
                selectPlanAction()
            }
            .accessibilityAction {
                selectPlanAction()
            }
#if os(iOS)
            .confirmationDialog(
                "createAccount.success.choosePlan".localizedString,
                isPresented: $isPlanAlertDisplayed,
                titleVisibility: .visible
            ) {
                ForEach(purchasesManager.products, id: \.id) { plan in
                    Button(subscriptionTitle(for: plan)) {
                        purchasePlan(with: plan)
                    }
                }
                Button("cancel".localizedString, role: .cancel) {}
            }
#endif
    }

#if os(iOS)
    var webSubscribeButton: some View {
        Button {
            beginWebSubscriptionPurchase()
        } label: {
            Text("subscriptionPurchase.choice.web".localizedString)
                .nymTextStyle(.bodyDefault)
                .foregroundStyle(Color.Nym.primary)
        }
        .accessibilityLabel("subscriptionPurchase.choice.web".localizedString)
    }
#endif
}

// MARK: - Actions -
extension GeneratePassphraseView {
    func navigateToPaymentSuccessView() {
        didLeaveForSuccessfulPurchase = true
        path.append(SettingLink.processingAccount)
    }

#if os(iOS)
    func beginWebSubscriptionPurchase() {
        autologinState.start(kind: .autologinRenew, using: credentialsManager)
    }
#endif
}
