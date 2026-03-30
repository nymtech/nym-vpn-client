import SwiftUI
import UIComponents
import Theme

// MARK: - Dialog Configurations -
extension AccountAndDevicesView {
    var logoutDialogConfiguration: ActionDialogConfiguration {
        ActionDialogConfiguration(
            systemIconImageName: "rectangle.portrait.and.arrow.right",
            titleLocalizedString: "settings.logoutTitle".localizedString,
            subtitleLocalizedString: "settings.logoutSubtitle".localizedString,
            yesLocalizedString: "settings.logout".localizedString,
            noLocalizedString: "cancel".localizedString,
            isYesDestructive: true,
            yesAction: {
                isLogoutLoading = true
                Task {
                    await logout()
                    try? await Task.sleep(for: .seconds(0.3))
                    Task { @MainActor in
                        isLogoutConfirmationDisplayed = false
                        isLogoutLoading = false
                        navigateBack()
                    }
                }
            },
            loadingText: "settings.loggingOut".localizedString,
            shouldCloseAfterYesAction: false,
            verticalButtonsLayout: true
        )
    }
}
