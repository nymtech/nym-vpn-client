import SwiftUI
import UIComponents
import Theme

// MARK: - Dialog Configurations -
extension AccountAndDevicesView {
    var autologinLoadingConfiguration: ActionDialogConfiguration {
        ActionDialogConfiguration(
            titleLocalizedString: "settings.account.autologin.fetchingPinCode".localizedString,
            noLocalizedString: "cancel".localizedString,
            noAction: {
                autologinTask?.cancel()
                autologinTask = nil
                isAutologinLoading = false
            },
            loadingText: "settings.account.autologin.loading".localizedString,
            shouldCloseAfterYesAction: false
        )
    }

    var autologinErrorConfiguration: ActionDialogConfiguration {
        ActionDialogConfiguration(
            systemIconImageName: "exclamationmark.triangle",
            systemIconImageColor: NymColor.error,
            titleLocalizedString: "generalNymError.somethingWentWrong".localizedString,
            subtitleLocalizedString: autologinErrorMessage,
            yesLocalizedString: "settings.account.autologin.retry".localizedString,
            noLocalizedString: "cancel".localizedString,
            yesAction: {
                isAutologinError = false
                navigateToAccount()
            },
            shouldCloseAfterYesAction: false,
            verticalButtonsLayout: true
        )
    }

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
