import Foundation
import os
import StoreKit

/// Supplies the Nym account's free trial verdicts. The app only carries them into StoreKit; it decides nothing.
@MainActor public protocol NymTrialEligibilityProviding: AnyObject {
    /// Whether a trial label may be shown. Only `true` means eligible; a failure is `false`.
    func isNymTrialEligible() async -> Bool

    /// Runs `body` with lookups bound to one account controller. Throws when the lookups cannot be opened.
    func withIntroOfferEligibilityLookups<T: Sendable>(
        _ body: @Sendable (IntroOfferEligibilityLookups) async -> T
    ) async throws -> T
}

/// The account lookups the purchase needs, as closures so the decision can be tested without the lib.
public struct IntroOfferEligibilityLookups: Sendable {
    public let isTrialEligible: @Sendable () async throws -> Bool
    public let checkAppleIntroOfferEligibility: @Sendable (_ productId: String, _ transactionId: String) async throws -> String?

    public init(
        isTrialEligible: @escaping @Sendable () async throws -> Bool,
        checkAppleIntroOfferEligibility: @escaping @Sendable (String, String) async throws -> String?
    ) {
        self.isTrialEligible = isTrialEligible
        self.checkAppleIntroOfferEligibility = checkAppleIntroOfferEligibility
    }
}

public enum IntroOfferEligibilityDecision: Equatable, Sendable {
    /// The account may get a trial; Apple's own rule decides. The endpoint was not called.
    case nymEligible
    /// The backend denied a trial; pass the token as `introductoryOfferEligibility(compactJWS:)`.
    case denied(compactJWS: String)
    /// No verdict to carry; Apple's own rule decides.
    case noToken(NoTokenReason)

    public enum NoTokenReason: String, Equatable, Sendable {
        case appTransactionUnavailable
        case endpointReturnedNoToken
        case endpointFailed
        case lookupsUnavailable
        case timedOut
    }

    /// Safe to log: never contains the token.
    public var logDescription: String {
        switch self {
        case .nymEligible:
            return "nym eligible, endpoint not called, token absent"
        case .denied:
            return "nym not eligible, token present"
        case let .noToken(reason):
            return "token absent (\(reason.rawValue))"
        }
    }
}

public enum IntroOfferEligibilityResolver {
    /// Bound on all eligibility work before a purchase. On timeout the purchase goes ahead without the option.
    public static let timeout: Duration = .seconds(5)

    /// The spec forbids sending `appTransactionID` for an eligible account, so `appTransactionID` and the
    /// endpoint are only touched when `isTrialEligible` did not return `true`.
    public static func decide(
        productId: String,
        lookups: IntroOfferEligibilityLookups,
        appTransactionID: @Sendable () async throws -> String?
    ) async -> IntroOfferEligibilityDecision {
        if (try? await lookups.isTrialEligible()) == true {
            return .nymEligible
        }
        guard let transactionId = try? await appTransactionID(), !transactionId.isEmpty else {
            return .noToken(.appTransactionUnavailable)
        }
        do {
            guard let compactJWS = try await lookups.checkAppleIntroOfferEligibility(productId, transactionId),
                  !compactJWS.isEmpty
            else {
                return .noToken(.endpointReturnedNoToken)
            }
            return .denied(compactJWS: compactJWS)
        } catch {
            return .noToken(.endpointFailed)
        }
    }

    public static func purchaseOptions(
        accountToken: UUID,
        decision: IntroOfferEligibilityDecision
    ) -> Set<Product.PurchaseOption> {
        guard case let .denied(compactJWS) = decision else {
            return [.appAccountToken(accountToken)]
        }
        return [.appAccountToken(accountToken), .introductoryOfferEligibility(compactJWS: compactJWS)]
    }

    /// Returns `fallback` once `timeout` passes, without waiting for `operation` to finish: the lib's async
    /// calls do not stop on task cancellation, so a structured race would still wait for them.
    public static func withTimeout<T: Sendable>(
        _ timeout: Duration,
        fallback: T,
        operation: @escaping @Sendable () async -> T
    ) async -> T {
        let isResumed = OSAllocatedUnfairLock(initialState: false)
        let claimResume: @Sendable () -> Bool = {
            isResumed.withLock { resumed in
                defer { resumed = true }
                return !resumed
            }
        }

        return await withCheckedContinuation { continuation in
            let work = Task {
                let value = await operation()
                if claimResume() {
                    continuation.resume(returning: value)
                }
            }
            Task {
                try? await Task.sleep(for: timeout)
                if claimResume() {
                    work.cancel()
                    continuation.resume(returning: fallback)
                }
            }
        }
    }
}
