import Combine
import SwiftUI
import UserNotifications
import AppSettings
import ConnectionManager

@MainActor public final class NotificationsManager: NSObject, ObservableObject {
    private let appSettings: AppSettings
    private let userNotificationCenter: UNUserNotificationCenter
    @ObservedObject private var connectionManager: ConnectionManager

    private var tunnelStatusUpdateCancellable: AnyCancellable?

    @Published public var permissionGranted = false

    public static let shared = NotificationsManager(appSettings: .shared, connectionManager: .shared)

    init(
        appSettings: AppSettings,
        connectionManager: ConnectionManager,
        userNotificationCenter: UNUserNotificationCenter = .current()
    ) {
        self.appSettings = appSettings
        self.connectionManager = connectionManager
        self.userNotificationCenter = userNotificationCenter
        super.init()
    }

    public func setup() {
        userNotificationCenter.delegate = self
        setupObservers()
        Task { [weak self] in
            await self?.checkNotificationPermission()
        }
    }
}

// MARK: - UNUserNotificationCenterDelegate
extension NotificationsManager: UNUserNotificationCenterDelegate {

    // Delegate methods are nonisolated; hop to MainActor as needed.

    nonisolated public func userNotificationCenter(
        _ center: UNUserNotificationCenter,
        didReceive response: UNNotificationResponse
    ) async {
        // handle on main if you need UI/state
        Task { @MainActor in
            // your handling here
            // print("🔥 \(response)")
        }
    }

    nonisolated public func userNotificationCenter(
        _ center: UNUserNotificationCenter,
        willPresent notification: UNNotification
    ) async -> UNNotificationPresentationOptions {

        // Read app active state on the main actor
        let isAppActive: Bool = await MainActor.run {
#if os(iOS)
            UIApplication.shared.applicationState == .active
#elseif os(macOS)
            NSApplication.shared.isActive
#endif
        }

        return isAppActive ? [] : [.badge, .banner, .sound]
    }
}

// MARK: - Setup
private extension NotificationsManager {
    func setupObservers() {
        tunnelStatusUpdateCancellable = connectionManager.$currentTunnelStatus
            .debounce(for: .seconds(0.3), scheduler: DispatchQueue.global(qos: .background))
            .removeDuplicates()
            .sink { [weak self] status in
                guard status == .connected else { return }
                Task { @MainActor [weak self] in
                    await self?.askForPermissionIfNeeded()
                }
            }
    }
}

// MARK: - Permissions
private extension NotificationsManager {
    func checkNotificationPermission() async {
        let settings = await userNotificationCenter.notificationSettings()
        switch settings.authorizationStatus {
        case .authorized, .provisional:
            permissionGranted = true
        default:
            permissionGranted = false
        }
    }

    func requestNotificationPermission() {
#if os(iOS)
        let options: UNAuthorizationOptions = [.alert, .badge, .sound]
#elseif os(macOS)
        let options: UNAuthorizationOptions = [.alert, .badge, .sound, .provisional]
#endif

        userNotificationCenter.requestAuthorization(options: options) { [weak self] granted, _ in
            Task { @MainActor [weak self] in
                self?.permissionGranted = granted
            }
        }
    }

    func askForPermissionIfNeeded() async {
        let status = await userNotificationCenter.notificationSettings().authorizationStatus
        guard status == .notDetermined else { return }
        requestNotificationPermission()
    }
}
