import Combine
import SwiftUI
import UserNotifications
import AppSettings
import ConnectionManager

final public class NotificationsManager: NSObject, ObservableObject {
    private let appSettings: AppSettings
    @ObservedObject private var connectionManager: ConnectionManager

    private var tunnelStatusUpdateCancellable: AnyCancellable?

    @Published public var permissionGranted = false

    public static let shared = NotificationsManager()

    init(
        appSettings: AppSettings = AppSettings.shared,
        connectionManager: ConnectionManager = ConnectionManager.shared
    ) {
        self.appSettings = appSettings
        self.connectionManager = connectionManager
    }

    public func setup() {
        UNUserNotificationCenter.current().delegate = self
        setupObservers()
        checkNotificationPermission()
    }
}

extension NotificationsManager: UNUserNotificationCenterDelegate {
    public func userNotificationCenter(
        _ center: UNUserNotificationCenter,
        willPresent notification: UNNotification
    ) async -> UNNotificationPresentationOptions {
        [.sound, .banner, .list]
    }
}

private extension NotificationsManager {
    func setupObservers() {
        tunnelStatusUpdateCancellable = connectionManager.$currentTunnelStatus
            .debounce(for: .seconds(0.3), scheduler: DispatchQueue.global(qos: .background))
            .removeDuplicates()
            .sink { [weak self] status in
                guard status == .connected else { return }
                self?.askForPermissionIfNeeded()
            }
    }
}

private extension NotificationsManager {
    func checkNotificationPermission() {
        UNUserNotificationCenter.current().getNotificationSettings { [weak self] settings in
            switch settings.authorizationStatus {
            case .notDetermined, .denied:
                self?.permissionGranted = false
            case .authorized, .provisional:
                self?.permissionGranted = true
            default:
                self?.permissionGranted = false
            }
        }
    }

    @MainActor func requestNotificationPermission() async {
        let result = try? await UNUserNotificationCenter.current().requestAuthorization(
            options: [.alert, .badge, .sound]
        )
        appSettings.didAskForNotificationPermission = true
        permissionGranted = result ?? false
    }

    func askForPermissionIfNeeded() {
        Task {
            guard !appSettings.didAskForNotificationPermission else { return }
            await requestNotificationPermission()
        }
    }
}
