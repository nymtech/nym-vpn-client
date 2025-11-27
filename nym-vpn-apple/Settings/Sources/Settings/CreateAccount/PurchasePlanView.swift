import SwiftUI
import StoreKit
import CredentialsManager
#if os(iOS)
import ImpactGenerator
import NymVPNLib
import ErrorHandler
#endif
import PurchasesManager
import Theme
import UIComponents

public struct PurchasePlanView: View {
    @EnvironmentObject private var credentialsManager: CredentialsManager
    @EnvironmentObject private var purchasesManager: PurchasesManager
    @Binding private var path: NavigationPath
    @State private var alertTitle = ""
    @State private var isPlanAlertDisplayed = false
    @State private var isAlertDisplayed = false
    @State private var isPurchasing = false
    @State private var shouldDisplayBackButton = false

    public var body: some View {
        VStack(alignment: .center, spacing: 0) {
            navbar
            Spacer()
                .frame(height: 24)

            StepView(stepCount: 2, currentStep: 1)
            Spacer()
            checkmarkImage
            Spacer()
                .frame(height: 12)
            titleSubtitleView
            Spacer()
            selectPlanButton
        }
        .frame(maxWidth: MagicNumbers.moreMaxWidth)
        .padding(16)
        .navigationBarBackButtonHidden(true)
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
        .alert(alertTitle, isPresented: $isAlertDisplayed) {
            Button("ok".localizedString, role: .cancel) {}
        }
    }

    public init(path: Binding<NavigationPath>, shouldDisplayBackButton: Bool) {
        _path = path
        self.shouldDisplayBackButton = shouldDisplayBackButton
    }
}

private extension PurchasePlanView {
    var navbar: some View {
        CustomNavBar(
            useElevationBackground: true,
            leftButton: CustomNavBarButton(
                type: .back,
                action: {
                    path = .init()
                }
            )
        )
    }

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
        VStack {
            Text("purchasePlan.title".localizedString)
                .textStyle(.Headline.Large.regular)
                .foregroundStyle(NymColor.primary)
                .multilineTextAlignment(.center)

            Spacer()
                .frame(height: 24)

            Text("purchasePlan.subtitle".localizedString)
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.gray1)
                .multilineTextAlignment(.center)

            Spacer()
                .frame(height: 24)

            Text("purchasePlan.subtitle2".localizedString)
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.gray1)
                .multilineTextAlignment(.center)
        }
    }

    var selectPlanButton: some View {
        GenericButton(title: "purchasePlan.selectPlan".localizedString, isLoading: $isPurchasing)
            .onTapGesture {
                selectPlanAction()
            }
            .accessibilityAction {
                selectPlanAction()
            }
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
    }
}

private extension PurchasePlanView {
    func subscriptionTitle(for plan: Product) -> String {
        if purchasesManager.isEligibleForIntroOffer.contains(plan.id),
           let subscription = plan.subscription,
           let offer = subscription.introductoryOffer {
            let periodDescription = offer.period.localizedDescription
            let offerText: String

            if offer.price == 0 {
                offerText = "\(periodDescription) free trial"
            } else {
                offerText = "\(offer.displayPrice) for \(periodDescription)"
            }
            return "\(plan.displayName) (\(plan.displayPrice), \(offerText))"
        } else {
            return "\(plan.displayName) (\(plan.displayPrice))"
        }
    }

    func selectPlanAction() {
        isPlanAlertDisplayed = true
    }

    func purchasePlanAction(with plan: Product) async {
        defer {
            isPurchasing = false
        }
        isPurchasing = true
#if os(iOS)
        ImpactGenerator.shared.impact()
#endif
        do {
            guard let token = credentialsManager.accountToken
            else {
                try await credentialsManager.registerAccount()
                return
            }
            let didPurchaseSuccesfully = try await purchasesManager.purchase(
                with: plan,
                token: token
            )
            guard didPurchaseSuccesfully else { return }
            navigateToPaymentSuccessView()
        } catch {
            Task { @MainActor in
#if os(iOS)
                if let lastVPNError = error as? VpnError {
                    alertTitle = VPNErrorReason(with: lastVPNError).errorDescription ?? ""
                } else {
                    alertTitle = error.localizedDescription
                }
                isAlertDisplayed = true
#endif
            }
        }
    }

    func navigateToPaymentSuccessView() {
        path.append(SettingLink.processingAccount)
    }
}

private extension Product.SubscriptionPeriod {
    var localizedDescription: String {
        let unitName: String
        switch unit {
        case .day:
            unitName = "day".localizedString
        case .week:
            unitName = "week".localizedString
        case .month:
            unitName = "month".localizedString
        case .year:
            unitName = "year".localizedString
        @unknown default:
            unitName = "period".localizedString
        }
        return "\(value)-\(unitName)"
    }
}
