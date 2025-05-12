import Logging
import UserNotifications
import Theme

public struct NotificationMessages {
    public static let disconnectNotificationIdentifier = "disconnectNotification"
    private static let lastScheduleKey = "disconnectNotification.lastScheduleDate"

    public static func scheduleDisconnectNotification() async {
        let now = Date()
        let defaults = UserDefaults.standard

        if let last = defaults.object(forKey: lastScheduleKey) as? Date,
           now.timeIntervalSince(last) < 10 * 60 {
            return
        }
        defaults.set(now, forKey: lastScheduleKey)

        let content = UNMutableNotificationContent()
        content.title = "notification.disconnected.title".localizedFromMainApp
        content.body  = "notification.disconnected.subtitle".localizedFromMainApp
        content.sound = .default

        let request = UNNotificationRequest(
            identifier: disconnectNotificationIdentifier,
            content: content,
            trigger: nil
        )

        do {
            try await UNUserNotificationCenter.current().add(request)
        } catch {
            Logger(label: "NotificationMessages")
                .info("Notification scheduling failed \(error)")
        }
    }
}
