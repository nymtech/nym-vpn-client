import Testing
import AccountPrefetchGates
@testable import Home

struct ProcessingAccountTitleBlockTests {
    @Test func titleBlockMode_prefetchBeforeSetupComplete_keepsSetupCarousel() {
        let showsCredentials = LoginProcessingCarouselVisibilityPolicy.showsCredentialsCopy(
            usesStaticCopy: false,
            didShowFinalMessage: false,
            isSyncing: false,
            isPrefetching: true,
            didFinishSetupCarousel: false
        )
        #expect(!showsCredentials)
        #expect(
            ProcessingAccountView.titleBlockMode(
                usesStaticCopy: false,
                didShowFinalMessage: false,
                showsCredentialsCarousel: showsCredentials
            ) == .setupCarousel
        )
    }

    @Test func titleBlockMode_syncing_showsCredentials() {
        let showsCredentials = LoginProcessingCarouselVisibilityPolicy.showsCredentialsCopy(
            usesStaticCopy: false,
            didShowFinalMessage: false,
            isSyncing: true,
            isPrefetching: false,
            didFinishSetupCarousel: true
        )
        #expect(showsCredentials)
        #expect(
            ProcessingAccountView.titleBlockMode(
                usesStaticCopy: false,
                didShowFinalMessage: false,
                showsCredentialsCarousel: showsCredentials
            ) == .credentials
        )
    }

    @Test func titleBlockMode_syncingAfterSetup_keepsSetupCarousel() {
        let showsCredentials = LoginProcessingCarouselVisibilityPolicy.showsCredentialsCopy(
            usesStaticCopy: false,
            didShowFinalMessage: false,
            isSyncing: false,
            isPrefetching: false
        )
        #expect(!showsCredentials)
        #expect(
            ProcessingAccountView.titleBlockMode(
                usesStaticCopy: false,
                didShowFinalMessage: false,
                showsCredentialsCarousel: showsCredentials
            ) == .setupCarousel
        )
    }

    @Test func titleBlockMode_setupUntilBackendWait() {
        #expect(
            ProcessingAccountView.titleBlockMode(
                usesStaticCopy: false,
                didShowFinalMessage: false,
                showsCredentialsCarousel: false
            ) == .setupCarousel
        )
    }

    @Test func titleBlockMode_welcomeAfterFinalize() {
        #expect(
            ProcessingAccountView.titleBlockMode(
                usesStaticCopy: false,
                didShowFinalMessage: true,
                showsCredentialsCarousel: false
            ) == .welcome
        )
    }
}
