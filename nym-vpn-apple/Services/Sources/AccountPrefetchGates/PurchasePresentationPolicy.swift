import Foundation

/// Visibility rules for onboarding chrome on the Settings purchase screen.
public enum PurchasePresentationPolicy: Equatable, Sendable {
    public static func showsOnboardingProgressBar(
        isPurchaseOnly: Bool,
        didFinishAnimatingText: Bool,
        didRegisterAccount: Bool
    ) -> Bool {
        if isPurchaseOnly {
            return false
        }
        return !(didFinishAnimatingText && didRegisterAccount)
    }

    public static func showsPurchasePanel(
        isPurchaseOnly: Bool,
        didFinishAnimatingText: Bool,
        didRegisterAccount: Bool
    ) -> Bool {
        isPurchaseOnly || (didFinishAnimatingText && didRegisterAccount)
    }
}
