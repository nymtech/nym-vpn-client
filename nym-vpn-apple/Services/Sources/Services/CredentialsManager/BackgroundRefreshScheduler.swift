#if os(iOS)
import BackgroundTasks
import Foundation
import Logging
import AccountPrefetchGates

public enum BackgroundRefreshScheduler {
    public static let appRefreshIdentifier = "net.nymtech.vpn.refresh"

    private static let logger = Logger(label: "BackgroundRefreshScheduler")
    private static let earliestInterval: TimeInterval = 4 * 60 * 60

    public static func scheduleAppRefresh() {
        let request = BGAppRefreshTaskRequest(identifier: appRefreshIdentifier)
        request.earliestBeginDate = Date(timeIntervalSinceNow: earliestInterval)
        do {
            try BGTaskScheduler.shared.submit(request)
            logger.debug("Scheduled background app refresh \(appRefreshIdentifier)")
        } catch {
            let nsError = error as NSError
            if nsError.domain == BGTaskScheduler.errorDomain,
               nsError.code == BGTaskScheduler.Error.Code.tooManyPendingTaskRequests.rawValue {
                logger.debug("Background app refresh already scheduled \(appRefreshIdentifier)")
            } else {
                logger.error("Failed to schedule background app refresh: \(error.localizedDescription)")
            }
        }
    }

    @MainActor
    public static func runRefresh() async {
        scheduleAppRefresh()

        let credentials = CredentialsManager.shared
        let outcome = await AccountPrefetchOrchestrator.runBackgroundRefresh(
            isCredentialImported: credentials.isValidCredentialImported,
            processing: credentials,
            timeout: 25
        )

        switch outcome.skipReason {
        case .noCredential:
            logger.debug("Background refresh skipped: no credential imported")
        case .inactiveAfterSummarySync:
            logger.info(
                "Background refresh complete (summary synced; prefetch skipped for inactive account)"
            )
        case nil:
            if let prefetch = outcome.prefetchResult {
                logger.info(
                    "Background refresh complete prefetch=\(prefetch) ready=\(prefetch.isReady)"
                )
            }
        }
    }
}
#endif
