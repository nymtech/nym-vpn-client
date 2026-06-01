#if os(macOS)
import Foundation
import ServiceManagement

final class LaunchAtStartupManager: ObservableObject {
    @Published var isEnabled: Bool

    init() {
        isEnabled = SMAppService.mainApp.status == .enabled
    }

    func refresh() {
        let enabled = SMAppService.mainApp.status == .enabled
        if enabled != isEnabled {
            isEnabled = enabled
        }
    }

    func apply(_ enabled: Bool) {
        do {
            if enabled {
                if SMAppService.mainApp.status != .enabled {
                    try SMAppService.mainApp.register()
                }
            } else {
                if SMAppService.mainApp.status != .notRegistered {
                    try SMAppService.mainApp.unregister()
                }
            }
        } catch {
            // Registration failed — fall back to the real system state.
            refresh()
        }
    }
}
#endif
