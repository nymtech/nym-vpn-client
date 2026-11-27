import SwiftUI
import UIComponents
import Theme

extension SettingsViewModel {
    func helpAndDataSection() -> AppSettingsSection {
        AppSettingsSection(
            kind: .helpAndData,
            viewModels: [
                SettingsListItemViewModel(
                    accessory: .arrow,
                    title: "settings.supportAndFeedback".localizedString,
                    imageName: "support",
                    action: { [weak self] in
                        Task { @MainActor in
                            self?.navigate(to: .support)
                        }
                    }
                ),
                SettingsListItemViewModel(
                    accessory: .arrow,
                    title: "settings.privacyAndData".localizedString,
                    systemImageName: "lock.doc",
                    action: { [weak self] in
                        Task { @MainActor in
                            self?.navigate(to: .privacyAndData)
                        }
                    }
                ),
                SettingsListItemViewModel(
                    accessory: .arrow,
                    title: "legal".localizedString,
                    systemImageName: "doc.text",
                    action: { [weak self] in
                        Task { @MainActor in
                            self?.navigate(to: .legal)
                        }
                    }
                )
            ]
        )
    }
}
