import Foundation
import Testing
import AccountPrefetchGates

struct LoginProcessingPolicyTests {
    @Test func loginProcessingUsesAnimatedCarousel() {
        #expect(LoginProcessingUI.requiresCarousel)
        #expect(LoginProcessingUI.carouselKeys.count == 4)
        #expect(LoginProcessingUI.setupCarouselPairs().count == 3)
        #expect(LoginProcessingUI.credentialsCarouselKeys.count == 4)
        #expect(LoginProcessingUI.credentialsCarouselPairs().count == 2)
        #expect(ProcessingUIPolicy.showsOnboardingProgressBar(usesStaticCopy: false))
    }

    @Test func setupCarouselPairs_useDistinctSubtitles() {
        let pairs = LoginProcessingUI.setupCarouselPairs()
        let subtitles = pairs.map(\.1)
        #expect(Set(subtitles).count == 3)
        #expect(subtitles.allSatisfy { !$0.isEmpty })
    }

    @Test func setupCarouselTiming_allowsReadableDwellPerStep() {
        #expect(LoginProcessingUI.setupCarouselInitialDwell >= 2)
        #expect(LoginProcessingUI.setupCarouselTickInterval >= 2)
        #expect(LoginProcessingUI.setupCarouselStepAdvanceDelay >= 1.5)
        #expect(LoginProcessingCarouselTimingPolicy.setupCarouselMinimumDurationSeconds() >= 7)
    }

    @Test func setupCarouselTextLeadsStepBar_byConfiguredDelay() {
        #expect(
            LoginProcessingCarouselTimingPolicy.textAdvancePrecedesStepBarTick(),
            "SwitchingTitlesView advances copy before timerDidTick when stepAdvanceDelay exceeds step bar pause"
        )
    }

    @Test func credentialsStepTwoCopy_doesNotPromiseConnect() throws {
        let title = try NymVPNXCStringsReader.englishValue(for: LoginProcessingUI.almostReadyTitleKey)
        let subtitle = try NymVPNXCStringsReader.englishValue(for: LoginProcessingUI.almostReadySubtitleKey)
        let combined = (title + " " + subtitle).lowercased()
        for term in LoginProcessingCopyPolicy.credentialsStepTwoForbiddenTerms {
            #expect(!combined.contains(term), "Credentials step 2 must not contain \(term)")
        }
    }

    @Test func credentialsCarouselVisibility_afterSetupDuringBackendWait() {
        #expect(
            !LoginProcessingCarouselVisibilityPolicy.showsCredentialsCopy(
                usesStaticCopy: false,
                didShowFinalMessage: false,
                didFinishSetupCarousel: false,
                isSyncing: true,
                isPrefetching: false,
                isPreparing: false
            )
        )
        #expect(
            LoginProcessingCarouselVisibilityPolicy.showsCredentialsCopy(
                usesStaticCopy: false,
                didShowFinalMessage: false,
                didFinishSetupCarousel: true,
                isSyncing: true,
                isPrefetching: false,
                isPreparing: false
            )
        )
        #expect(
            LoginProcessingCarouselVisibilityPolicy.showsCredentialsCopy(
                usesStaticCopy: false,
                didShowFinalMessage: false,
                didFinishSetupCarousel: true,
                isSyncing: false,
                isPrefetching: false,
                isPreparing: true
            )
        )
        #expect(
            LoginProcessingCarouselVisibilityPolicy.showsCredentialsCopy(
                usesStaticCopy: false,
                didShowFinalMessage: false,
                didFinishSetupCarousel: true,
                isSyncing: false,
                isPrefetching: true,
                isPreparing: false
            )
        )
        #expect(
            !LoginProcessingCarouselVisibilityPolicy.showsCredentialsCopy(
                usesStaticCopy: true,
                didShowFinalMessage: false,
                didFinishSetupCarousel: true,
                isSyncing: true,
                isPrefetching: true,
                isPreparing: true
            )
        )
    }

    @Test func credentialsCarouselPairKeys_tick0_and_tick1() {
        let tick0 = LoginProcessingCopyPolicy.credentialsCarouselPairKeys(tickIndex: 0)
        #expect(tick0.title == LoginProcessingUI.loadingCredentialsTitleKey)
        #expect(tick0.subtitle == LoginProcessingUI.loadingCredentialsSubtitleKey)

        let tick1 = LoginProcessingCopyPolicy.credentialsCarouselPairKeys(tickIndex: 1)
        #expect(tick1.title == LoginProcessingUI.almostReadyTitleKey)
        #expect(tick1.subtitle == LoginProcessingUI.almostReadySubtitleKey)

        let tickOverflow = LoginProcessingCopyPolicy.credentialsCarouselPairKeys(tickIndex: 99)
        #expect(tickOverflow.title == LoginProcessingUI.almostReadyTitleKey)
        #expect(tickOverflow.subtitle == LoginProcessingUI.almostReadySubtitleKey)
    }

    @Test func loginCarouselBlocksNavigationUntilAnimationCompletes() {
        #expect(
            !ProcessingAccountReadiness.canAdvanceNavigation(
                didCompleteAccountPrep: true,
                didFinishAnimatingText: false,
                requiresCarousel: true
            )
        )
        #expect(
            ProcessingAccountReadiness.canAdvanceNavigation(
                didCompleteAccountPrep: true,
                didFinishAnimatingText: true,
                requiresCarousel: true
            )
        )
    }
}
