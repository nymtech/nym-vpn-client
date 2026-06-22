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

struct LoginProcessingOrchestratorTests {
    @Test func loginProcessingRunsImportPrepThenPrefetch() async throws {
        final class EventLog: @unchecked Sendable {
            private(set) var events: [LoginProcessingOrchestrator.Step] = []

            func record(_ step: LoginProcessingOrchestrator.Step) {
                events.append(step)
            }
        }

        let log = EventLog()

        _ = try await LoginProcessingOrchestrator.run(
            ensureCredentialImportResolved: {
                log.record(.credentialImportResolved)
            },
            prepareRegisteredAccount: {
                log.record(.accountPrepared)
            },
            runProcessingFlow: {
                log.record(.processingFlowCompleted)
                return AccountPrefetchOrchestrator.ProcessingOutcome(
                    didSyncSummary: true,
                    prefetchResult: .fetchedTickets
                )
            }
        )

        #expect(
            log.events == [
                .credentialImportResolved,
                .accountPrepared,
                .processingFlowCompleted
            ]
        )
    }

    @Test func loginProcessingPrepFailureSkipsPrefetch() async {
        final class EventLog: @unchecked Sendable {
            var importResolved = false
            var prefetchCalled = false
        }

        let log = EventLog()

        await #expect(throws: TestPrepError.self) {
            try await LoginProcessingOrchestrator.run(
                ensureCredentialImportResolved: {
                    log.importResolved = true
                },
                prepareRegisteredAccount: {
                    throw TestPrepError.failed
                },
                runProcessingFlow: {
                    log.prefetchCalled = true
                    return AccountPrefetchOrchestrator.ProcessingOutcome(
                        didSyncSummary: true,
                        prefetchResult: .fetchedTickets
                    )
                }
            )
        }

        #expect(log.importResolved)
        #expect(!log.prefetchCalled)
    }
}

private enum TestPrepError: Error {
    case failed
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
