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
        super.init()
    }

    public func setup() {
        UNUserNotificationCenter.current().delegate = self

        setupObservers()
        checkNotificationPermission()
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
                self?.askForPermissionIfNeeded()
            }
    }
}

// MARK: - Permissions -
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

    func requestNotificationPermission() {
        UNUserNotificationCenter.current().requestAuthorization(
            options: [.alert, .badge, .sound]
        ) { [weak self] granted, _ in
            self?.permissionGranted = granted
        }
    }

    func askForPermissionIfNeeded() {
        guard !appSettings.didAskForNotificationPermission else { return }
        Task { @MainActor in
            appSettings.didAskForNotificationPermission = true
        }
        requestNotificationPermission()
    }
}
