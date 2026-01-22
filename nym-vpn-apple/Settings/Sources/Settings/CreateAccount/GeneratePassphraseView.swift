import SwiftUI
import StoreKit
import AppSettings
import CredentialsManager
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
    @State var alertTitle: String = ""
    @State var didRegisterAccount = false
    @State var isAlertDisplayed = false
    @State var isPurchasing = false
    @State var isPlanAlertDisplayed = false

    @EnvironmentObject var appSettings: AppSettings
    @EnvironmentObject var credentialsManager: CredentialsManager

    public var body: some View {
        VStack(spacing: 0) {
            navbar()
            Spacer()
                .frame(height: 24)

            StepView(stepCount: 4, currentStep: $currentStep)
            Spacer()

            if !didFinishAnimatingText {
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
        .padding(16)
        .navigationBarBackButtonHidden(true)
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
        .task {
            await generateAndRegisterMnemonic()
        }
        .alert(alertTitle, isPresented: $isAlertDisplayed) {
            Button("retry".localizedString, role: .cancel) {
                Task {
                    await generateAndRegisterMnemonic()
                }
            }
        }
    }

    public init(path: Binding<NavigationPath>, displayPurchaseView: Bool = false) {
        _path = path
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
                RoundedRectangle(cornerRadius: 8)
                    .fill(Color(red: 0.07, green: 0.77, blue: 0.37).opacity(0.15))
                    .frame(width: 68, height: 68)
                    .overlay(
                        RoundedRectangle(cornerRadius: 8)
                            .stroke(Color(red: 0.08, green: 0.91, blue: 0.44).opacity(0.25), lineWidth: 1)
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
                .textStyle(.Headline.Large.regular)
                .foregroundStyle(NymColor.primary)
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
            subscriptionBenefitCell(with: "allFeaturesIncluded", title: "purchasePlan.allFeatures".localizedString)
            subscriptionBenefitCell(with: "noAds", title: "purchasePlan.noAds".localizedString)
            subscriptionBenefitCell(with: "cancelAnytime", title: "purchasePlan.cancelAnytime".localizedString)
        }
    }

    func subscriptionBenefitCell(with imageName: String, title: String) -> some View {
        HStack(spacing: 8) {
            GenericImage(imageName: imageName)
                .foregroundStyle(NymColor.accent)
                .frame(width: 24, height: 24)
            Text(title)
                .foregroundStyle(NymColor.gray1)
                .textStyle(.Body.Medium.regular)
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
                        Task {
                            await purchasePlanAction(with: plan)
                        }
                    }
                }
                Button("cancel".localizedString, role: .cancel) {}
            }
#endif
    }
}

// MARK: - Actions -
extension GeneratePassphraseView {
    func navigateToPaymentSuccessView() {
        path.append(SettingLink.processingAccount)
    }
}
