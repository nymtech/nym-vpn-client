import Foundation

public enum ZkNymPrefetchResult: Equatable, Sendable {
    case sufficientBandwidth
    case fetchedTickets
    case upgradeMode
    case skippedStoreBusy
    case skipped
    case failed

    /// True when local zk-nyms are sufficient or were fetched successfully.
    public var isReady: Bool {
        switch self {
        case .sufficientBandwidth, .fetchedTickets, .upgradeMode:
            return true
        case .skippedStoreBusy, .skipped, .failed:
            return false
        }
    }
}

/// Pure gates for when the app should prefetch zk-nyms (processing, background refresh).
public enum AccountZkNymPrefetchGate: Equatable, Sendable {
    public enum BackgroundRefreshPlan: Equatable, Sendable {
        case skipNoCredential
        case syncSummaryOnly
        case syncAndPrefetch
    }

    /// Prefetch only when the freshly synced summary reports an active subscription.
    public static func shouldPrefetchAfterSummarySync(isAccountActive: Bool) -> Bool {
        isAccountActive
    }

    /// Plan after summary sync (call with post-sync `isAccountActive`).
    public static func postSummarySyncPlan(isAccountActive: Bool) -> BackgroundRefreshPlan {
        isAccountActive ? .syncAndPrefetch : .syncSummaryOnly
    }

    /// Pre-sync plan: credential guard only; active status must be read after summary sync.
    public static func backgroundRefreshPlan(
        isCredentialImported: Bool,
        isAccountActiveAfterSummarySync: Bool
    ) -> BackgroundRefreshPlan {
        guard isCredentialImported else { return .skipNoCredential }
        return postSummarySyncPlan(isAccountActive: isAccountActiveAfterSummarySync)
    }
}

/// Shared async sequencing for processing screens and background refresh.
public enum AccountPrefetchOrchestrator: Sendable {
    public struct ProcessingOutcome: Equatable, Sendable {
        public let didSyncSummary: Bool
        public let prefetchResult: ZkNymPrefetchResult?

        public init(didSyncSummary: Bool, prefetchResult: ZkNymPrefetchResult?) {
            self.didSyncSummary = didSyncSummary
            self.prefetchResult = prefetchResult
        }
    }

    public enum BackgroundSkipReason: Equatable, Sendable {
        case noCredential
        case inactiveAfterSummarySync
    }

    public struct BackgroundOutcome: Equatable, Sendable {
        public let skipReason: BackgroundSkipReason?
        public let didSyncSummary: Bool
        public let prefetchResult: ZkNymPrefetchResult?

        public init(
            skipReason: BackgroundSkipReason?,
            didSyncSummary: Bool,
            prefetchResult: ZkNymPrefetchResult?
        ) {
            self.skipReason = skipReason
            self.didSyncSummary = didSyncSummary
            self.prefetchResult = prefetchResult
        }
    }

    public static func runProcessingFlow(
        isAccountActive: @Sendable () async -> Bool,
        updateAccountSummary: @Sendable () async -> Void,
        prefetchZkNyms: @Sendable () async -> ZkNymPrefetchResult
    ) async -> ProcessingOutcome {
        await updateAccountSummary()
        guard shouldPrefetchAfterSummarySync(isAccountActive: await isAccountActive()) else {
            return ProcessingOutcome(didSyncSummary: true, prefetchResult: nil)
        }
        let prefetch = await prefetchZkNyms()
        return ProcessingOutcome(didSyncSummary: true, prefetchResult: prefetch)
    }

    public static func runBackgroundRefresh(
        isCredentialImported: Bool,
        isAccountActive: @Sendable () async -> Bool,
        updateAccountSummary: @Sendable () async -> Void,
        prefetchZkNyms: @Sendable () async -> ZkNymPrefetchResult
    ) async -> BackgroundOutcome {
        guard isCredentialImported else {
            return BackgroundOutcome(
                skipReason: .noCredential,
                didSyncSummary: false,
                prefetchResult: nil
            )
        }
        await updateAccountSummary()
        guard shouldPrefetchAfterSummarySync(isAccountActive: await isAccountActive()) else {
            return BackgroundOutcome(
                skipReason: .inactiveAfterSummarySync,
                didSyncSummary: true,
                prefetchResult: nil
            )
        }
        let prefetch = await prefetchZkNyms()
        return BackgroundOutcome(
            skipReason: nil,
            didSyncSummary: true,
            prefetchResult: prefetch
        )
    }
}

private extension AccountPrefetchOrchestrator {
    static func shouldPrefetchAfterSummarySync(isAccountActive: Bool) -> Bool {
        AccountZkNymPrefetchGate.shouldPrefetchAfterSummarySync(isAccountActive: isAccountActive)
    }
}

/// Processing screens wait for account prep and animation before navigating.
public enum ProcessingAccountReadiness: Equatable, Sendable {
    public static func canAdvanceNavigation(
        didCompleteAccountPrep: Bool,
        didFinishAnimatingText: Bool
    ) -> Bool {
        didCompleteAccountPrep && didFinishAnimatingText
    }
}
