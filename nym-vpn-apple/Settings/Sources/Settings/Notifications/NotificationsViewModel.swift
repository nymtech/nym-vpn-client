import SwiftUI
import AppSettings
import ConnectionManager

@MainActor
public final class NotificationsViewModel: ObservableObject {
    @Binding private var path: NavigationPath
    private let appSettings: AppSettings
    private let connectionManager: ConnectionManager

    @Published var isServerFamilyRemindersEnabled: Bool

    public init(
        path: Binding<NavigationPath>,
        appSettings: AppSettings,
        connectionManager: ConnectionManager
    ) {
        _path = path
        self.appSettings = appSettings
        self.connectionManager = connectionManager
        self.isServerFamilyRemindersEnabled = appSettings.serverFamilyRemindersEnabled
    }

    func navigateBack() {
        guard !path.isEmpty else { return }
        path.removeLast()
    }

    func setServerFamilyReminders(_ enabled: Bool) {
        isServerFamilyRemindersEnabled = enabled
        connectionManager.setGatewayIndependenceNotifications(enabled)
    }
}
