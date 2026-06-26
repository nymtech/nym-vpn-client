import Foundation

public enum LoginProcessingUI: Sendable {
    public static let progressStep = 4
    public static let initialProgressStep = 2
    public static let requiresCarousel = true
    public static let carouselStepRange = 2...4
    public static let carouselTitlePrefix = "processingAccount.login"
    public static let settingUpTitleKey = "\(carouselTitlePrefix).settingUp"

    public static var carouselKeys: [String] {
        [settingUpTitleKey]
    }
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
