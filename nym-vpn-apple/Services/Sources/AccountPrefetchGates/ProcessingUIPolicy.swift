import Foundation

public enum LoginProcessingUI: Sendable {
    public static let progressStep = 4
    public static let initialProgressStep = 2
    public static let requiresCarousel = true
    public static let carouselStepRange = 2...4
    public static let carouselTitlePrefix = "processingAccount.login"
    public static let settingUpTitleKey = "\(carouselTitlePrefix).settingUp"
    public static let settingUpStep2SubtitleKey = "\(carouselTitlePrefix).settingUpStep2Subtitle"
    public static let settingUpStep3SubtitleKey = "\(carouselTitlePrefix).settingUpStep3Subtitle"
    public static let settingUpStep4SubtitleKey = "\(carouselTitlePrefix).settingUpStep4Subtitle"
    public static let setupCarouselInitialDwell: TimeInterval = 2.5
    public static let setupCarouselTickInterval: TimeInterval = 2.5
    public static let setupCarouselStepAdvanceDelay: TimeInterval = 2.0
    public static let stepBarInitialLeadIn: TimeInterval = 0.8
    public static let stepBarStepPause: TimeInterval = 1.0
    public static let loadingCredentialsTitleKey = "\(carouselTitlePrefix).loadingCredentials"
    public static let loadingCredentialsSubtitleKey = "\(carouselTitlePrefix).loadingCredentialsSubtitle"
    public static let almostReadyTitleKey = "\(carouselTitlePrefix).almostReady"
    public static let almostReadySubtitleKey = "\(carouselTitlePrefix).almostReadySubtitle"
    public static let credentialsCarouselTickInterval: TimeInterval = 10
    public static let credentialsCarouselStepCount = 2

    public static var carouselKeys: [String] {
        [
            settingUpTitleKey,
            settingUpStep2SubtitleKey,
            settingUpStep3SubtitleKey,
            settingUpStep4SubtitleKey,
        ]
    }

    public static func setupCarouselPairs() -> [(String, String)] {
        let title = settingUpTitleKey.localizedString
        return [
            (title, settingUpStep2SubtitleKey.localizedString),
            (title, settingUpStep3SubtitleKey.localizedString),
            (title, settingUpStep4SubtitleKey.localizedString),
        ]
    }

    public static var credentialsCarouselKeys: [String] {
        [
            loadingCredentialsTitleKey,
            loadingCredentialsSubtitleKey,
            almostReadyTitleKey,
            almostReadySubtitleKey,
        ]
    }

    public static func credentialsCarouselPairs() -> [(String, String)] {
        [
            (loadingCredentialsTitleKey.localizedString, loadingCredentialsSubtitleKey.localizedString),
            (almostReadyTitleKey.localizedString, almostReadySubtitleKey.localizedString),
        ]
    }
}

public enum LoginProcessingCarouselTimingPolicy: Sendable {
    public static func setupCarouselMinimumDurationSeconds() -> TimeInterval {
        let stepCount = LoginProcessingUI.setupCarouselPairs().count
        guard stepCount > 0 else { return 0 }
        return LoginProcessingUI.setupCarouselInitialDwell
            + TimeInterval(stepCount - 1) * LoginProcessingUI.setupCarouselTickInterval
    }

    public static func textAdvancePrecedesStepBarTick() -> Bool {
        LoginProcessingUI.setupCarouselStepAdvanceDelay > LoginProcessingUI.stepBarStepPause
    }
}

public enum LoginProcessingCarouselVisibilityPolicy: Sendable {
    public static func showsCredentialsCopy(
        usesStaticCopy: Bool,
        didShowFinalMessage: Bool,
        didFinishSetupCarousel: Bool,
        isSyncing: Bool,
        isPrefetching: Bool,
        isPreparing: Bool
    ) -> Bool {
        guard !usesStaticCopy, !didShowFinalMessage, didFinishSetupCarousel else { return false }
        return isPreparing || isSyncing || isPrefetching
    }
}

public enum LoginProcessingCopyPolicy: Sendable {
    public static let credentialsStepTwoForbiddenTerms = ["connect", "ready"]

    public static func credentialsCarouselPairKeys(tickIndex: Int) -> (title: String, subtitle: String) {
        switch max(0, tickIndex) {
        case 0:
            return (
                LoginProcessingUI.loadingCredentialsTitleKey,
                LoginProcessingUI.loadingCredentialsSubtitleKey
            )
        default:
            return (
                LoginProcessingUI.almostReadyTitleKey,
                LoginProcessingUI.almostReadySubtitleKey
            )
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
