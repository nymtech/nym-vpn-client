import Foundation

public enum LoginProcessingUI: Sendable {
    public static let progressStep = 4
    public static let initialProgressStep = 1
    /// Setup carousel fills bar segments 1-3; segment 4 is backend prefetch only.
    public static let setupCarouselMaxProgressStep = 3
    public static let requiresCarousel = true
    public static let carouselStepRange = 2...4
    public static let carouselTitlePrefix = "processingAccount.login"
    public static let settingUpTitleKey = "\(carouselTitlePrefix).settingUp"
    public static let settingUpStep2SubtitleKey = "\(carouselTitlePrefix).settingUpStep2Subtitle"
    public static let settingUpStep3SubtitleKey = "\(carouselTitlePrefix).settingUpStep3Subtitle"
    public static let settingUpStep4SubtitleKey = "\(carouselTitlePrefix).settingUpStep4Subtitle"
    /// Log-calibrated on LTE login (~12:07:57-12:08:11): ~3s per readable beat before prefetch at ~14s.
    public static let setupCarouselInitialDwell: TimeInterval = 3.0
    public static let setupCarouselTickInterval: TimeInterval = 3.0
    /// Extra hold on the last setup subtitle before handoff to backend copy.
    public static let setupCarouselFinalPairDwell: TimeInterval = 3.5
    public static let stepBarInitialLeadIn: TimeInterval = 0.8
    public static let stepBarAnimateInitialFill = true
    public static let stepBarStepPause: TimeInterval = 1.0
    /// Text and bar advance on the same carousel tick (no lead/lag).
    public static let setupCarouselStepAdvanceDelay: TimeInterval = 0
    public static let setupCarouselTextTransitionDuration: TimeInterval = 0.35
    public static let loadingCredentialsTitleKey = "\(carouselTitlePrefix).loadingCredentials"
    public static let loadingCredentialsSubtitleKey = "\(carouselTitlePrefix).loadingCredentialsSubtitle"
    public static let almostReadyTitleKey = "\(carouselTitlePrefix).almostReady"
    public static let almostReadySubtitleKey = "\(carouselTitlePrefix).almostReadySubtitle"
    /// Upper bound for `prefetchZkNyms`; typical device prefetch is 15-30s.
    public static let prefetchTimeoutSeconds: TimeInterval = 60
    public static var carouselKeys: [String] {
        [
            settingUpTitleKey,
            settingUpStep2SubtitleKey,
            settingUpStep3SubtitleKey,
            settingUpStep4SubtitleKey,
        ]
    }

    public static func setupCarouselPairs() -> [(String, String)] {
        let context = settingUpTitleKey.localizedString
        return [
            (settingUpStep2SubtitleKey.localizedString, context),
            (settingUpStep3SubtitleKey.localizedString, context),
            (settingUpStep4SubtitleKey.localizedString, context),
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
            + LoginProcessingUI.setupCarouselFinalPairDwell
    }

    public static func usesUnifiedSegmentDwell() -> Bool {
        LoginProcessingUI.setupCarouselStepAdvanceDelay == LoginProcessingUI.stepBarStepPause
    }

    public static func textAdvanceSyncsWithStepBarTick() -> Bool {
        LoginProcessingUI.setupCarouselStepAdvanceDelay == 0
    }

    public static func textAdvancePrecedesStepBarTick() -> Bool {
        LoginProcessingUI.setupCarouselStepAdvanceDelay > LoginProcessingUI.stepBarStepPause
    }
}

public enum LoginProcessingBackendPhasePolicy: Sendable {
    public enum DisplayPhase: Equatable, Sendable {
        case syncing
        case prefetching
    }

    public static func displayPhase(
        for accountPhase: OnboardingAccountPreparationPolicy.AccountStatePhase
    ) -> DisplayPhase? {
        switch accountPhase {
        case .syncing:
            return .syncing
        case .requestingZkNyms:
            return .prefetching
        case .offline, .loggedOut, .readyToConnect, .decentralised, .upgradeMode,
             .pendingSubscription, .error:
            return nil
        }
    }
}

public enum LoginProcessingProgressPolicy: Sendable {
    public static func setupProgressStep(carouselIndex: Int) -> Int {
        let index = max(0, carouselIndex)
        return min(index + 1, LoginProcessingUI.setupCarouselMaxProgressStep)
    }

    public static func credentialsCopyKeys(
        isSyncing: Bool,
        isPrefetching: Bool,
        holdsPrefetchCopyThroughAdvance: Bool = false,
        didFinishSetupCarousel: Bool = false
    ) -> (title: String, subtitle: String)? {
        guard didFinishSetupCarousel else { return nil }
        if isPrefetching || holdsPrefetchCopyThroughAdvance {
            return (
                LoginProcessingUI.almostReadyTitleKey,
                LoginProcessingUI.almostReadySubtitleKey
            )
        }
        guard isSyncing else { return nil }
        return (
            LoginProcessingUI.loadingCredentialsTitleKey,
            LoginProcessingUI.loadingCredentialsSubtitleKey
        )
    }

    public static func progressStep(
        setupCarouselIndex: Int,
        didFinishSetupCarousel: Bool,
        isPrefetching: Bool,
        isAwaitingAdvance: Bool,
        hasReachedPrefetchPhase: Bool = false
    ) -> Int {
        // Lockstep: bars 1-3 follow setup copy. Prefetch must not skip to 4 mid-carousel.
        if didFinishSetupCarousel && (isPrefetching || isAwaitingAdvance || hasReachedPrefetchPhase) {
            return LoginProcessingUI.progressStep
        }
        if didFinishSetupCarousel {
            return LoginProcessingUI.setupCarouselMaxProgressStep
        }
        return setupProgressStep(carouselIndex: setupCarouselIndex)
    }
}

public enum LoginProcessingCarouselVisibilityPolicy: Sendable {
    public static func showsCredentialsCopy(
        usesStaticCopy: Bool,
        didShowFinalMessage: Bool,
        isSyncing: Bool,
        isPrefetching: Bool,
        holdsPrefetchCopyThroughAdvance: Bool = false,
        didFinishSetupCarousel: Bool = false
    ) -> Bool {
        guard !usesStaticCopy, !didShowFinalMessage, didFinishSetupCarousel else { return false }
        return isSyncing || isPrefetching || holdsPrefetchCopyThroughAdvance
    }
}

public enum LoginProcessingCopyPolicy: Sendable {
    public static let credentialsStepTwoForbiddenTerms = ["connect", "ready"]
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
