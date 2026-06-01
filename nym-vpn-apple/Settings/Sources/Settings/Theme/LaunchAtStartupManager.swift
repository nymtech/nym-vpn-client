#if os(macOS)
import Foundation
import os
import ServiceManagement

final class LaunchAtStartupManager: ObservableObject {
    @Published var isEnabled = false
    @Published var lastError: String?

    private let logger = Logger(
        subsystem: Bundle.main.bundleIdentifier ?? "NymVPN",
        category: "LaunchAtStartup"
    )

    init() {
        refresh()
    }

    func refresh() {
        let status = SMAppService.mainApp.status
        isEnabled = status == .enabled || status == .requiresApproval
    }

    func apply(_ enabled: Bool) {
        lastError = nil
        do {
            switch SMAppService.mainApp.status {
            case .notRegistered:
                if enabled {
                    try SMAppService.mainApp.register()
                }
            case .enabled, .requiresApproval:
                if !enabled {
                    try SMAppService.mainApp.unregister()
                }
            default:
                break
            }
        } catch {
            lastError = error.localizedDescription
            logger.error(
                "Login item \(enabled ? "register" : "unregister", privacy: .public) failed: \(error.localizedDescription, privacy: .public)"
            )
        }
        refresh()
        if enabled, SMAppService.mainApp.status == .requiresApproval {
            SMAppService.openSystemSettingsLoginItems()
        }
    }
}
#endif
