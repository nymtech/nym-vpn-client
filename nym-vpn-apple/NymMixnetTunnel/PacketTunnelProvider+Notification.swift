// MARK: - Notifications -
import UserNotifications

extension PacketTunnelProvider {
    func scheduleDisconnectNotification() {
        // TODO: localize the notification content.
        // TODO: move localizations to separate package
        let content = UNMutableNotificationContent()
        content.title = "The NymVPN connection has failed."
        content.body = "Please try reconnecting."
        content.sound = UNNotificationSound.default

        let request = UNNotificationRequest(identifier: "disconnectNotification", content: content, trigger: nil)

        UNUserNotificationCenter.current().add(request) { [weak self] error in
            if let error = error {
                print("Error scheduling notification: \(error.localizedDescription)")
            } else {
                self?.logger.info("🚀 Notification scheduled successfully")
            }
        }
    }
}
