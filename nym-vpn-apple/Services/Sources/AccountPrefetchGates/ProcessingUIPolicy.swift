import Foundation

public enum LoginProcessingUI: Sendable {
    public static let progressStep = 4
    public static let requiresCarousel = false
    public static let titleKey = "processingAccount.loginSync.title"
    public static let subtitleKey = "processingAccount.loginSync.subtitle"
}

public enum PostPurchaseProcessingUI: Sendable {
    public static let progressStep = 4
    public static let requiresCarousel = false
    public static let titleKey = "processingAccount.awaitingConfirmation.title"
    public static let subtitleKey = "processingAccount.awaitingConfirmation.subtitle"
}

public enum ProcessingUIPolicy: Equatable, Sendable {
    public static func showsOnboardingProgressBar(usesStaticCopy: Bool) -> Bool {
        !usesStaticCopy
    }
}
