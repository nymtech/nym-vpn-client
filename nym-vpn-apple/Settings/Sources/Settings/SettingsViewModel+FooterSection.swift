import SwiftUI
import UIComponents
import Theme

extension SettingsViewModel {
    /// Log out while signed in; the app version is a block drawn by SettingsView.
    func footerSection() -> AppSettingsSection {
        var viewModels = [SettingsListItemViewModel]()
        if appSettings.isCredentialImported {
            viewModels.append(
                SettingsListItemViewModel(
                    accessory: .empty,
                    title: "settings.logout".localizedString,
                    systemImageName: "rectangle.portrait.and.arrow.right",
                    action: { [weak self] in
                        Task { @MainActor in
                            self?.impactGenerator.softImpact()
                            self?.isLogoutConfirmationDisplayed = true
                        }
                    }
                )
            )
        }
        return AppSettingsSection(kind: .footer, viewModels: viewModels)
    }
}
