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
        #expect(
            LoginProcessingProgressPolicy.credentialsCopyKeys(isSyncing: false, isPrefetching: false) == nil
        )
    }

    @Test func credentialsCopyKeys_syncingAndPrefetch() {
        #expect(LoginProcessingProgressPolicy.credentialsCopyKeys(isSyncing: false, isPrefetching: false) == nil)

        let syncing = LoginProcessingProgressPolicy.credentialsCopyKeys(
            isSyncing: true,
            isPrefetching: false,
            didFinishSetupCarousel: true
        )
        #expect(syncing?.title == LoginProcessingUI.loadingCredentialsTitleKey)
        #expect(syncing?.subtitle == LoginProcessingUI.loadingCredentialsSubtitleKey)

        let prefetching = LoginProcessingProgressPolicy.credentialsCopyKeys(
            isSyncing: false,
            isPrefetching: true,
            didFinishSetupCarousel: true
        )
        #expect(prefetching?.title == LoginProcessingUI.almostReadyTitleKey)
        #expect(prefetching?.subtitle == LoginProcessingUI.almostReadySubtitleKey)

        #expect(
            LoginProcessingProgressPolicy.credentialsCopyKeys(
                isSyncing: true,
                isPrefetching: true,
                didFinishSetupCarousel: true
            )?.title == LoginProcessingUI.almostReadyTitleKey
        )
        let awaitingAfterPrefetch = LoginProcessingProgressPolicy.credentialsCopyKeys(
            isSyncing: false,
            isPrefetching: false,
            holdsPrefetchCopyThroughAdvance: true,
            didFinishSetupCarousel: true
        )
        #expect(awaitingAfterPrefetch?.title == LoginProcessingUI.almostReadyTitleKey)
    }

    @Test func backendPhasePolicy_mapsControllerPhasesToDisplay() {
        #expect(
            LoginProcessingBackendPhasePolicy.displayPhase(for: .syncing) == .syncing
        )
        #expect(
            LoginProcessingBackendPhasePolicy.displayPhase(for: .requestingZkNyms) == .prefetching
        )
        #expect(LoginProcessingBackendPhasePolicy.displayPhase(for: .readyToConnect) == nil)
        #expect(LoginProcessingBackendPhasePolicy.displayPhase(for: .offline) == nil)
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

    @Test func progressStep_prefetchDuringSetup_staysOnCarouselSegment() {
        #expect(
            LoginProcessingProgressPolicy.progressStep(
                setupCarouselIndex: 0,
                didFinishSetupCarousel: false,
                isPrefetching: true,
                isAwaitingAdvance: false,
                hasReachedPrefetchPhase: true
            ) == 1
        )
        #expect(
            LoginProcessingProgressPolicy.progressStep(
                setupCarouselIndex: 1,
                didFinishSetupCarousel: false,
                isPrefetching: true,
                isAwaitingAdvance: false,
                hasReachedPrefetchPhase: true
            ) == 2
        )
        #expect(
            LoginProcessingProgressPolicy.progressStep(
                setupCarouselIndex: 2,
                didFinishSetupCarousel: false,
                isPrefetching: true,
                isAwaitingAdvance: false,
                hasReachedPrefetchPhase: true
            ) == 3
        )
    }

    @Test func progressStep_fourthSegmentOnlyAfterSetupFinishes() {
        #expect(
            LoginProcessingProgressPolicy.progressStep(
                setupCarouselIndex: 0,
                didFinishSetupCarousel: true,
                isPrefetching: true,
                isAwaitingAdvance: false,
                hasReachedPrefetchPhase: true
            ) == 4
        )
    }

    @Test func credentialsCopy_hiddenUntilSetupCarouselFinishes() {
        #expect(
            LoginProcessingProgressPolicy.credentialsCopyKeys(
                isSyncing: true,
                isPrefetching: true,
                didFinishSetupCarousel: false
            ) == nil
        )
        #expect(
            !LoginProcessingCarouselVisibilityPolicy.showsCredentialsCopy(
                usesStaticCopy: false,
                didShowFinalMessage: false,
                isSyncing: true,
                isPrefetching: true,
                didFinishSetupCarousel: false
            )
        )
    }

    @Test func credentialsCopy_defaultHidesUntilSetupFinishes() {
        #expect(
            LoginProcessingProgressPolicy.credentialsCopyKeys(
                isSyncing: true,
                isPrefetching: true
            ) == nil
        )
        #expect(
            !LoginProcessingCarouselVisibilityPolicy.showsCredentialsCopy(
                usesStaticCopy: false,
                didShowFinalMessage: false,
                isSyncing: true,
                isPrefetching: true
            )
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

    @Test func credentialsCarouselVisibility_syncingAndPrefetch() {
        #expect(
            !LoginProcessingCarouselVisibilityPolicy.showsCredentialsCopy(
                usesStaticCopy: false,
                didShowFinalMessage: false,
                isSyncing: false,
                isPrefetching: false
            )
        )
        #expect(
            LoginProcessingCarouselVisibilityPolicy.showsCredentialsCopy(
                usesStaticCopy: false,
                didShowFinalMessage: false,
                isSyncing: true,
                isPrefetching: false,
                didFinishSetupCarousel: true
            )
        )
        #expect(
            LoginProcessingCarouselVisibilityPolicy.showsCredentialsCopy(
                usesStaticCopy: false,
                didShowFinalMessage: false,
                isSyncing: false,
                isPrefetching: true,
                didFinishSetupCarousel: true
            )
        )
        #expect(
            !LoginProcessingCarouselVisibilityPolicy.showsCredentialsCopy(
                usesStaticCopy: true,
                didShowFinalMessage: false,
                isSyncing: true,
                isPrefetching: true
            )
        )
        #expect(
            !LoginProcessingCarouselVisibilityPolicy.showsCredentialsCopy(
                usesStaticCopy: false,
                didShowFinalMessage: true,
                isSyncing: true,
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
