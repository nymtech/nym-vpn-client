import Logging
import UserNotifications

public struct NotificationMessages {
    public static func scheduleDisconnectNotification() async {
        // TODO: localize the notification content.
        // TODO: move localizations to separate package
        let content = UNMutableNotificationContent()
        content.title = "The NymVPN connection has failed."
        content.body = "Please try reconnecting."
        content.sound = UNNotificationSound.default

        let request = UNNotificationRequest(identifier: "disconnectNotification", content: content, trigger: nil)

        do {
            try await UNUserNotificationCenter.current().add(request)
        } catch {
            Logger(label: "NotificationMessages").info("🚀 Notification scheduled successfully")
        }
    }
}
