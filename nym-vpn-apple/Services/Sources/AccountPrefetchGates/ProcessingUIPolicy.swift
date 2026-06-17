import Foundation

public enum LoginProcessingUI: Sendable {
    public static let progressStep = 4
    public static let initialProgressStep = 2
    public static let requiresCarousel = true
    public static let carouselStepRange = 2...4
    public static let carouselTitlePrefix = "processingAccount.login"

    public static var carouselKeys: [String] {
        carouselStepRange.flatMap { step in
            ["\(carouselTitlePrefix).title\(step)", "\(carouselTitlePrefix).subtitle\(step)"]
        }
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

public enum AuthFlowHeightPolicy: Equatable, Sendable {
    public static func sharedRootHeight(
        welcome: CGFloat,
        signUp: CGFloat,
        signIn: CGFloat,
        passphrase: CGFloat,
        generateCarousel: CGFloat
    ) -> CGFloat {
        max(welcome, signUp, signIn, passphrase, generateCarousel)
    }

    /// Sign-in must not inherit signup carousel measurement height.
    public static func signInRootHeight(
        welcome: CGFloat,
        signIn: CGFloat,
        passphrase: CGFloat
    ) -> CGFloat {
        max(welcome, signIn, passphrase)
    }
}
