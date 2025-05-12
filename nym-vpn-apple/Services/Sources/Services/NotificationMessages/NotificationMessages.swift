import Logging
import UserNotifications
import Theme

public struct NotificationMessages {
    public static let disconnectNotificationIdentifier = "disconnectNotification"

    public static func scheduleDisconnectNotification() async {
        let content = UNMutableNotificationContent()
        content.title = "notification.disconnected.title".localizedFromMainApp
        content.body = "notification.disconnected.subtitle".localizedFromMainApp
        content.sound = UNNotificationSound.default

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
