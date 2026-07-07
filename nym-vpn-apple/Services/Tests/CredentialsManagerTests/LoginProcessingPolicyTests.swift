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

    @Test func setupCarouselPairs_useDistinctHeadlines() {
        let pairs = LoginProcessingUI.setupCarouselPairs()
        let headlines = pairs.map(\.0)
        #expect(Set(headlines).count == 3)
        #expect(headlines.allSatisfy { !$0.isEmpty })
    }

    @Test func setupCarouselPairs_sharesContextSubtitle() {
        let pairs = LoginProcessingUI.setupCarouselPairs()
        let subtitles = pairs.map(\.1)
        #expect(Set(subtitles).count == 1)
        #expect(!subtitles[0].isEmpty)
    }

    @Test func setupCarouselTiming_allowsReadableDwellPerStep() {
        #expect(LoginProcessingUI.setupCarouselInitialDwell >= 3)
        #expect(LoginProcessingUI.setupCarouselTickInterval >= 3)
        #expect(LoginProcessingUI.setupCarouselFinalPairDwell >= 3)
        #expect(LoginProcessingUI.setupCarouselStepAdvanceDelay >= 0)
        #expect(LoginProcessingCarouselTimingPolicy.setupCarouselMinimumDurationSeconds() >= 12)
    }

    @Test func setupCarouselTiming_textSyncsWithStepBar() {
        #expect(LoginProcessingCarouselTimingPolicy.textAdvanceSyncsWithStepBarTick())
        #expect(!LoginProcessingCarouselTimingPolicy.textAdvancePrecedesStepBarTick())
        #expect(LoginProcessingUI.setupCarouselStepAdvanceDelay == 0)
        #expect(LoginProcessingUI.setupCarouselTextTransitionDuration > 0)
    }

    @Test func setupProgressStep_mapsEachCarouselIndex() {
        #expect(LoginProcessingProgressPolicy.setupProgressStep(carouselIndex: 0) == 1)
        #expect(LoginProcessingProgressPolicy.setupProgressStep(carouselIndex: 1) == 2)
        #expect(LoginProcessingProgressPolicy.setupProgressStep(carouselIndex: 2) == 3)
    }

    @Test func stepBar_animatesInitialFillToFirstSegment() {
        #expect(LoginProcessingUI.stepBarAnimateInitialFill)
        #expect(LoginProcessingUI.initialProgressStep == 1)
    }

    @Test func progressStep_holdsThirdSegmentAfterSetupUntilBackendPhase() {
        #expect(
            LoginProcessingProgressPolicy.progressStep(
                setupCarouselIndex: 2,
                didFinishSetupCarousel: true,
                isPrefetching: false,
                isAwaitingAdvance: false
            ) == 3
        )
        #expect(LoginProcessingProgressPolicy.credentialsCopyKeys(isPrefetching: false) == nil)
    }

    @Test func credentialsCopyKeys_prefetchOnly() {
        #expect(LoginProcessingProgressPolicy.credentialsCopyKeys(isPrefetching: false) == nil)

        let prefetching = LoginProcessingProgressPolicy.credentialsCopyKeys(isPrefetching: true)
        #expect(prefetching?.title == LoginProcessingUI.almostReadyTitleKey)
        #expect(prefetching?.subtitle == LoginProcessingUI.almostReadySubtitleKey)
    }

    @Test func prefetchTimeout_coversTypicalDeviceWait() {
        #expect(LoginProcessingUI.prefetchTimeoutSeconds >= 30)
    }

    @Test func progressStep_capsSetupAtThirdSegmentDuringCarousel() {
        #expect(
            LoginProcessingProgressPolicy.progressStep(
                setupCarouselIndex: 2,
                didFinishSetupCarousel: false,
                isPrefetching: false,
                isAwaitingAdvance: false
            ) == 3
        )
    }

    @Test func progressStep_holdsThirdSegmentDuringSync() {
        #expect(
            LoginProcessingProgressPolicy.progressStep(
                setupCarouselIndex: 2,
                didFinishSetupCarousel: true,
                isPrefetching: false,
                isAwaitingAdvance: false
            ) == 3
        )
    }

    @Test func progressStep_holdsFourthSegmentUntilPrefetchOrCompletion() {
        #expect(
            LoginProcessingProgressPolicy.progressStep(
                setupCarouselIndex: 2,
                didFinishSetupCarousel: true,
                isPrefetching: true,
                isAwaitingAdvance: false
            ) == 4
        )
        #expect(
            LoginProcessingProgressPolicy.progressStep(
                setupCarouselIndex: 2,
                didFinishSetupCarousel: true,
                isPrefetching: false,
                isAwaitingAdvance: true
            ) == 4
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

    @Test func credentialsCarouselVisibility_prefetchOnly() {
        #expect(
            !LoginProcessingCarouselVisibilityPolicy.showsCredentialsCopy(
                usesStaticCopy: false,
                didShowFinalMessage: false,
                isPrefetching: false
            )
        )
        #expect(
            LoginProcessingCarouselVisibilityPolicy.showsCredentialsCopy(
                usesStaticCopy: false,
                didShowFinalMessage: false,
                isPrefetching: true
            )
        )
        #expect(
            !LoginProcessingCarouselVisibilityPolicy.showsCredentialsCopy(
                usesStaticCopy: true,
                didShowFinalMessage: false,
                isPrefetching: true
            )
        )
        #expect(
            !LoginProcessingCarouselVisibilityPolicy.showsCredentialsCopy(
                usesStaticCopy: false,
                didShowFinalMessage: true,
                isPrefetching: true
            )
        )
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
