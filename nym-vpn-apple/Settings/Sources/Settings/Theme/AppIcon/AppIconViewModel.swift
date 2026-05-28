#if os(iOS)
import SwiftUI
import UIKit

@MainActor public final class AppIconViewModel: ObservableObject {
    let title = "settings.appIcon.title".localizedString

    @Published public var selectedIcon: AppIcon
    @Published public var pendingIcon: AppIcon?

    var icons: [AppIcon] {
        AppIcon.allCases
    }

    public init(path: Binding<NavigationPath>) {
        _path = path
        // UIApplication.shared.alternateIconName is the single source of truth
        // and persists across launches.
        let alternateName = UIApplication.shared.alternateIconName
        self.selectedIcon = AppIcon.allCases.first { $0.alternateName == alternateName } ?? .default
    }

    @Binding var path: NavigationPath

    func requestChange(to icon: AppIcon) {
        guard icon != selectedIcon else { return }
        pendingIcon = icon
    }

    func cancelChange() {
        pendingIcon = nil
    }

    func confirmChange() async {
        guard let icon = pendingIcon else { return }
        do {
            try await UIApplication.shared.setAlternateIconName(icon.alternateName)
            selectedIcon = icon
        } catch {
            // Change rejected (e.g. in simulator with restrictions); dismiss silently
        }
        pendingIcon = nil
    }
}

// MARK: - Navigation
extension AppIconViewModel {
    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }
}
#endif
