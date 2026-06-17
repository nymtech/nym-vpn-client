#if os(iOS)
import StoreKit
import ImpactGenerator
import ErrorHandler
import NymVPNLib

// MARK: - Views -
extension GeneratePassphraseView {
    func subscriptionTitle(for plan: Product) -> String {
        if purchasesManager.isEligibleForIntroOffer.contains(plan.id),
           let subscription = plan.subscription,
           let offer = subscription.introductoryOffer {
            let periodDescription = offer.period.localizedDescription
            let offerText: String

            if offer.price == 0 {
                offerText = "\("incl".localizedString) \(periodDescription) \("freeTrial".localizedString)"
            } else {
                offerText = "\(offer.displayPrice) for \(periodDescription)"
            }
            return "\(plan.displayName) (\(plan.displayPrice)) \(offerText)"
        } else {
            return "\(plan.displayName) (\(plan.displayPrice))"
        }
    }
}

// MARK: - Actions -
extension GeneratePassphraseView {
    func generateAndRegisterMnemonic() async {
        guard !isRegistering else { return }
        isRegistering = true
        do {
            try await credentialsManager.performAccountRegistration()
            didRegisterAccount = true
            isRegistering = false
        } catch {
            Task { @MainActor in
                alertTitle = registrationErrorDescription(error)
                isAlertDisplayed = true
                didRegisterAccount = false
                isRegistering = false
            }
            return
        }
    }

    func purchasePlanAction(with plan: Product) async {
        defer {
            isPurchasing = false
        }
        isPurchasing = true
        ImpactGenerator.shared.impact()

        do {
            guard let token = credentialsManager.accountToken, !token.isEmpty else {
                alertTitle = "accountToken.empty".localizedString
                isAlertDisplayed = true
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
                alertTitle = registrationErrorDescription(error)
                isAlertDisplayed = true
            }
        }
    }

    func purchasePlan(with plan: Product) {
        guard let accountToken = credentialsManager.accountToken,
              !accountToken.isEmpty
        else {
            alertTitle = "accountToken.empty".localizedString
            isAlertDisplayed = true
            return
        }
        Task {
            await purchasePlanAction(with: plan)
        }
    }

    func selectPlanAction() {
        isPlanAlertDisplayed = true
    }

    func registrationErrorDescription(_ error: Error) -> String {
        if let reason = error as? VPNErrorReason {
            return reason.errorDescription ?? ""
        }
        if let vpnError = error as? VpnError {
            return VPNErrorReason(with: vpnError).errorDescription ?? ""
        }
        return error.localizedDescription
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
#endif
