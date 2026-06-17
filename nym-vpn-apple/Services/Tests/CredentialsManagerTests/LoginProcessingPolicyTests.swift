import Foundation
import Testing
import AccountPrefetchGates

struct AuthFlowHeightPolicyTests {
    @Test func signInHeightExcludesGenerateCarouselMeasurement() {
        let welcome: CGFloat = 400
        let signIn: CGFloat = 420
        let passphrase: CGFloat = 450
        let generateCarousel: CGFloat = 700

        let shared = AuthFlowHeightPolicy.sharedRootHeight(
            welcome: welcome,
            signUp: 500,
            signIn: signIn,
            passphrase: passphrase,
            generateCarousel: generateCarousel
        )
        let signInOnly = AuthFlowHeightPolicy.signInRootHeight(
            welcome: welcome,
            signIn: signIn,
            passphrase: passphrase
        )

        #expect(shared == generateCarousel)
        #expect(signInOnly == passphrase)
        #expect(signInOnly < shared)
    }
}

struct LoginProcessingPolicyTests {
    @Test func loginProcessingUsesAnimatedCarousel() {
        #expect(LoginProcessingUI.requiresCarousel)
        #expect(LoginProcessingUI.carouselKeys.count == 6)
        #expect(ProcessingUIPolicy.showsOnboardingProgressBar(usesStaticCopy: false))
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
