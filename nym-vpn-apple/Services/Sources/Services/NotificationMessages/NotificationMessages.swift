import Logging
import UserNotifications
import Theme

public struct NotificationMessages {
    public static func scheduleDisconnectNotification() async {
        let content = UNMutableNotificationContent()
        content.title = "notifcation.disconnected.title".localizedString
        content.body = "notifcation.disconnected.subtitle ".localizedString
        content.sound = UNNotificationSound.default

        let request = UNNotificationRequest(identifier: "disconnectNotification", content: content, trigger: nil)

        do {
            try await UNUserNotificationCenter.current().add(request)
        } catch {
            Logger(label: "NotificationMessages").info("🚀 Notification scheduled successfully")
        }
    }
}
