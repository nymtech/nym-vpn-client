#if os(iOS)
import StoreKit
import ImpactGenerator
import ErrorHandler
import NymVPNLib
import AccountPrefetchGates
import PurchasesManager

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
                alertOffersRegistrationRetry = true
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
            try await credentialsManager.ensureAccountRegisteredForCurrentEnvironment()
        } catch {
            presentPurchaseAlert(message: registrationErrorDescription(error))
            return
        }
        do {
            guard let token = credentialsManager.accountToken, !token.isEmpty else {
                presentPurchaseAlert(
                    message: "accountToken.empty".localizedString
                )
                return
            }
            let outcome = try await purchasesManager.purchase(
                with: plan,
                token: token
            )
            let checkoutResult = mapPurchaseOutcome(outcome)
            switch checkoutResult {
            case .success:
                navigateToPaymentSuccessView()
            case .userCancelled, .pending, .failed:
                presentPurchaseAlert(
                    message: IAPFeedbackPolicy.alertLocalizationKey(for: checkoutResult).localizedString
                )
            }
        } catch {
            Task { @MainActor in
                presentPurchaseAlert(
                    message: IAPFeedbackPolicy.alertLocalizationKey(for: .failed).localizedString
                )
            }
        }
    }

    func presentPurchaseAlert(message: String) {
        alertOffersRegistrationRetry = false
        alertTitle = message
        isAlertDisplayed = true
    }

    func purchasePlan(with plan: Product) {
        guard let accountToken = credentialsManager.accountToken,
              !accountToken.isEmpty
        else {
            presentPurchaseAlert(message: "accountToken.empty".localizedString)
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

    func mapPurchaseOutcome(_ outcome: PurchaseOutcome) -> IAPCheckoutResult {
        switch outcome {
        case .success:
            return .success
        case .userCancelled:
            return .userCancelled
        case .pending:
            return .pending
        case .failed:
            return .failed
        }
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
