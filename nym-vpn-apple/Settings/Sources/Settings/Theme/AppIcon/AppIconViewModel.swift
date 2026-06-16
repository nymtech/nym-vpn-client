import SwiftUI
import SnackbarManager
import Theme

@MainActor
final class AppIconViewModel: ObservableObject {
    let title = "settings.appIcon".localizedString

    @Published private(set) var current: AppIcon
    @Binding var path: NavigationPath

    private let changer: AppIconChanging

    init(path: Binding<NavigationPath> = .constant(NavigationPath()), changer: AppIconChanging) {
        _path = path
        self.changer = changer
        self.current = AppIcon(alternateName: changer.currentAlternateIconName)
    }

    var icons: [AppIcon] { AppIcon.allCases }

    func select(_ icon: AppIcon) async {
        guard icon != current else { return }
        do {
            try await changer.setAlternateIconName(icon.alternateName)
            current = AppIcon(alternateName: changer.currentAlternateIconName)
        } catch {
            SnackbarManager.shared.enqueue(
                SnackbarItem(
                    style: .critical,
                    title: "appIcon.error".localizedString
                )
            )
        }
    }

    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }
}
