#if os(iOS)
import BackgroundTasks
import Foundation
import Logging

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
            logger.error("Failed to schedule background app refresh: \(error.localizedDescription)")
        }
    }

    @MainActor
    public static func runRefresh() async {
        scheduleAppRefresh()

        let credentials = CredentialsManager.shared
        guard credentials.isValidCredentialImported else {
            logger.debug("Background refresh skipped: no credential imported")
            return
        }

        await credentials.updateAccountSummary(force: true)
        let prefetch = await credentials.prefetchZkNyms()
        logger.info("Background refresh complete prefetch=\(prefetch)")
    }
}
#endif
