import SwiftUI
import UIComponents
import Theme

extension SettingsViewModel {
    func appSection() -> AppSettingsSection {
        AppSettingsSection(
            kind: .app,
            viewModels: [
                SettingsListItemViewModel(
                    accessory: .arrow,
                    title: "settings.appearance".localizedString,
                    imageName: "appearance",
                    action: { [weak self] in
                        Task { @MainActor in
                            self?.navigate(to: .appearance)
                        }
                    }
                ),
                SettingsListItemViewModel(
                    accessory: .arrow,
                    title: "settings.notifications.title".localizedString,
                    systemImageName: "bell",
                    action: { [weak self] in
                        Task { @MainActor in
                            self?.navigate(to: .notifications)
                        }
                    }
                )
            ]
        )
    }
}
