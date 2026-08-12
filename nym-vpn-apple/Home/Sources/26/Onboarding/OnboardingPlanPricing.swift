#if os(iOS)
import PurchasesManager
#endif

/// Pricing shown on the onboarding plan screen.
/// iOS derives it from StoreKit; macOS purchases happen on the web, so it uses the Figma values.
struct OnboardingPlanPricing: Equatable {
    let monthlyPrice: String
    let savings: String?
    let freeTrialPeriod: String?

#if os(macOS)
    @MainActor
    init?() {
        monthlyPrice = Constants.monthlyPrice
        savings = Constants.savings
        freeTrialPeriod = Constants.freeTrialPeriod
    }
#else
    @MainActor
    init?(purchasesManager: PurchasesManager) {
        guard let monthlyPrice = purchasesManager.yearlyPlanMonthlyPriceText else { return nil }
        self.monthlyPrice = monthlyPrice
        savings = purchasesManager.yearlyPlanSavingsText
        freeTrialPeriod = purchasesManager.yearlyPlanFreeTrialPeriodText
    }
#endif
}

#if os(macOS)
private extension OnboardingPlanPricing {
    /// Figma 4440-20189 — the web plan has no StoreKit product to read from.
    enum Constants {
        static let monthlyPrice = "$8.26"
        static let savings = "65%"
        static let freeTrialPeriod = "7-day"
    }
}
#endif
