import Combine
import SwiftUI
import UserNotifications
import AppSettings
import ConnectionManager

final public class NotificationsManager: NSObject, ObservableObject {
    private let appSettings: AppSettings
    private let userNotificationCenter: UNUserNotificationCenter
    @ObservedObject private var connectionManager: ConnectionManager

    private var tunnelStatusUpdateCancellable: AnyCancellable?

    @Published public var permissionGranted = false

    public static let shared = NotificationsManager()

    init(
        appSettings: AppSettings = AppSettings.shared,
        connectionManager: ConnectionManager = ConnectionManager.shared,
        userNotificationCenter: UNUserNotificationCenter = UNUserNotificationCenter.current()
    ) {
        self.appSettings = appSettings
        self.connectionManager = connectionManager
        self.userNotificationCenter = userNotificationCenter
        super.init()
    }

    public func setup() {
        userNotificationCenter.delegate = self

        setupObservers()
        Task {
            await checkNotificationPermission()
        }
    }
}

// MARK: - UNUserNotificationCenterDelegate -
extension NotificationsManager: UNUserNotificationCenterDelegate {
    public func userNotificationCenter(
        _ center: UNUserNotificationCenter,
        didReceive response: UNNotificationResponse
    ) async {
        print("🔥")
        print(response)
    }

    public func userNotificationCenter(
        _ center: UNUserNotificationCenter,
        willPresent notification: UNNotification
    ) async -> UNNotificationPresentationOptions {
        [.banner, .sound, .badge]
    }
}

// MARK: - Setup -
private extension NotificationsManager {
    func setupObservers() {
        tunnelStatusUpdateCancellable = connectionManager.$currentTunnelStatus
            .debounce(for: .seconds(0.3), scheduler: DispatchQueue.global(qos: .background))
            .removeDuplicates()
            .sink { [weak self] status in
                guard status == .connected else { return }
                Task {
                    await self?.askForPermissionIfNeeded()
                }
            }
    }
}

// MARK: - Permissions -
private extension NotificationsManager {
    func checkNotificationPermission() async {
        let settings = await userNotificationCenter.notificationSettings()
        switch settings.authorizationStatus {
        case .notDetermined, .denied:
            permissionGranted = false
        case .authorized, .provisional:
            permissionGranted = true
        default:
            permissionGranted = false
        }
    }

    func requestNotificationPermission() {
        userNotificationCenter.requestAuthorization(
            options: [.alert, .badge, .sound]
        ) { [weak self] granted, _ in
            self?.permissionGranted = granted
        }
    }

    func askForPermissionIfNeeded() async {
        let notificationAuthorizationStatus = await userNotificationCenter.notificationSettings().authorizationStatus
        guard notificationAuthorizationStatus == .notDetermined else { return }
        requestNotificationPermission()
    }
}