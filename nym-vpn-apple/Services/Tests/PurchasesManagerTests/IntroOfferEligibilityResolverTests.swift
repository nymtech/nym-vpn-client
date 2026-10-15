import Foundation
import os
import Testing
import PurchasesManager

struct IntroOfferEligibilityResolverTests {
    private struct LookupFailure: Error {}

    /// Counts calls so tests can prove the endpoint and `AppTransaction` were not touched.
    private final class CallLog: Sendable {
        private let counts = OSAllocatedUnfairLock(initialState: [String: Int]())

        func record(_ name: String) {
            counts.withLock { $0[name, default: 0] += 1 }
        }

        func count(_ name: String) -> Int {
            counts.withLock { $0[name] ?? 0 }
        }
    }

    private static let productId = "1_year_may_2025"
    private static let transactionId = "app-transaction-id"
    private static let token = "header.payload.signature"

    private func decide(
        isTrialEligible: @escaping @Sendable () async throws -> Bool,
        appTransactionID: @escaping @Sendable () async throws -> String? = { transactionId },
        endpoint: @escaping @Sendable (String, String) async throws -> String?,
        log: CallLog = CallLog()
    ) async -> IntroOfferEligibilityDecision {
        let lookups = IntroOfferEligibilityLookups(
            isTrialEligible: {
                log.record("isTrialEligible")
                return try await isTrialEligible()
            },
            checkAppleIntroOfferEligibility: { productId, transactionId in
                log.record("endpoint")
                return try await endpoint(productId, transactionId)
            }
        )
        return await IntroOfferEligibilityResolver.decide(
            productId: Self.productId,
            lookups: lookups,
            appTransactionID: {
                log.record("appTransaction")
                return try await appTransactionID()
            }
        )
    }

    // MARK: - Decision table

    @Test func eligible_skipsEndpointAndAppTransaction() async {
        let log = CallLog()
        let decision = await decide(
            isTrialEligible: { true },
            endpoint: { _, _ in Self.token },
            log: log
        )
        #expect(decision == .nymEligible)
        #expect(log.count("isTrialEligible") == 1)
        #expect(log.count("appTransaction") == 0)
        #expect(log.count("endpoint") == 0)
    }

    @Test(arguments: [false, true])
    func notEligibleOrThrew_tokenReturned_denies(isTrialEligibleThrows: Bool) async {
        let decision = await decide(
            isTrialEligible: {
                if isTrialEligibleThrows { throw LookupFailure() }
                return false
            },
            endpoint: { productId, transactionId in
                #expect(productId == Self.productId)
                #expect(transactionId == Self.transactionId)
                return Self.token
            }
        )
        #expect(decision == .denied(compactJWS: Self.token))
    }

    @Test(arguments: [false, true])
    func notEligibleOrThrew_endpointReturnsNil_noToken(isTrialEligibleThrows: Bool) async {
        let decision = await decide(
            isTrialEligible: {
                if isTrialEligibleThrows { throw LookupFailure() }
                return false
            },
            endpoint: { _, _ in nil }
        )
        #expect(decision == .noToken(.endpointReturnedNoToken))
    }

    @Test func notEligible_endpointThrows_noToken() async {
        let decision = await decide(
            isTrialEligible: { false },
            endpoint: { _, _ in throw LookupFailure() }
        )
        #expect(decision == .noToken(.endpointFailed))
    }

    @Test func notEligible_endpointReturnsEmptyToken_noToken() async {
        let decision = await decide(
            isTrialEligible: { false },
            endpoint: { _, _ in "" }
        )
        #expect(decision == .noToken(.endpointReturnedNoToken))
    }

    @Test func notEligible_appTransactionThrows_skipsEndpoint() async {
        let log = CallLog()
        let decision = await decide(
            isTrialEligible: { false },
            appTransactionID: { throw LookupFailure() },
            endpoint: { _, _ in Self.token },
            log: log
        )
        #expect(decision == .noToken(.appTransactionUnavailable))
        #expect(log.count("endpoint") == 0)
    }

    @Test func notEligible_appTransactionUnverified_skipsEndpoint() async {
        let log = CallLog()
        let decision = await decide(
            isTrialEligible: { false },
            appTransactionID: { nil },
            endpoint: { _, _ in Self.token },
            log: log
        )
        #expect(decision == .noToken(.appTransactionUnavailable))
        #expect(log.count("endpoint") == 0)
    }

    // MARK: - Purchase options

    @Test func purchaseOptions_carryTokenOnlyWhenDenied() {
        let accountToken = UUID()

        #expect(
            IntroOfferEligibilityResolver.purchaseOptions(accountToken: accountToken, decision: .nymEligible)
                == [.appAccountToken(accountToken)]
        )
        #expect(
            IntroOfferEligibilityResolver.purchaseOptions(accountToken: accountToken, decision: .noToken(.timedOut))
                == [.appAccountToken(accountToken)]
        )
        #expect(
            IntroOfferEligibilityResolver.purchaseOptions(
                accountToken: accountToken,
                decision: .denied(compactJWS: Self.token)
            ) == [.appAccountToken(accountToken), .introductoryOfferEligibility(compactJWS: Self.token)]
        )
    }

    @Test func logDescription_neverContainsToken() {
        #expect(!IntroOfferEligibilityDecision.denied(compactJWS: Self.token).logDescription.contains(Self.token))
    }

    // MARK: - Timeout

    @Test func timeout_returnsFallbackWithoutWaitingForWork() async {
        let clock = ContinuousClock()
        let start = clock.now
        let decision = await IntroOfferEligibilityResolver.withTimeout(
            .milliseconds(50),
            fallback: IntroOfferEligibilityDecision.noToken(.timedOut)
        ) {
            // Ignores cancellation, like the lib's async calls.
            await withCheckedContinuation { continuation in
                DispatchQueue.global().asyncAfter(deadline: .now() + 10) {
                    continuation.resume()
                }
            }
            return .denied(compactJWS: Self.token)
        }
        #expect(decision == .noToken(.timedOut))
        #expect(clock.now - start < .seconds(5))
    }

    @Test func timeout_returnsWorkResultWhenFastEnough() async {
        let decision = await IntroOfferEligibilityResolver.withTimeout(
            .seconds(5),
            fallback: IntroOfferEligibilityDecision.noToken(.timedOut)
        ) {
            .denied(compactJWS: Self.token)
        }
        #expect(decision == .denied(compactJWS: Self.token))
    }
}
