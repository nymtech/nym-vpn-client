import SwiftUI
import StoreKit
import AppSettings
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
    private let displayPurchaseView: Bool
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
#if os(macOS)
    @State var autologinState = AutologinState()
#endif

    @EnvironmentObject var appSettings: AppSettings
    @EnvironmentObject var credentialsManager: CredentialsManager

    public var body: some View {
        VStack(spacing: 0) {
            navbar()
            Spacer()
                .frame(height: 24)

            VStack(spacing: 0) {
                StepView(stepCount: 4, currentStep: $currentStep)
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
            if displayPurchaseView {
                didRegisterAccount = true
            } else {
                await generateAndRegisterMnemonic()
            }
        }
        .alert(alertTitle, isPresented: $isAlertDisplayed) {
            Button("retry".localizedString, role: .cancel) {
                Task {
                    await generateAndRegisterMnemonic()
                }
            }
        }
#if os(macOS)
        .autologinOverlay(state: autologinState)
#endif
    }

    public init(path: Binding<NavigationPath>, displayPurchaseView: Bool = false) {
        _path = path
        self.displayPurchaseView = displayPurchaseView
        didFinishAnimatingText = displayPurchaseView
        currentStep = displayPurchaseView ? 4 : 1
    }
}

// MARK: - Views -
private extension GeneratePassphraseView {
    @ViewBuilder
    func navbar() -> some View {
        if didFinishAnimatingText {
            CustomNavBar(
                useElevationBackground: true,
                rightButton: CustomNavBarButton(
                    type: .close,
                    action: {
                        dismissPurchaseFlow()
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
                Button("cancel".localizedString, role: .cancel) {
                    OnboardingSession.shared.cancelPurchaseFlow()
                }
            }
#endif
    }
}

// MARK: - Actions -
extension GeneratePassphraseView {
    func dismissPurchaseFlow() {
        OnboardingSession.shared.cancelPurchaseFlow()
        path = .init()
    }

    func navigateToPaymentSuccessView() {
        path.append(SettingLink.processingAccount)
    }
}
