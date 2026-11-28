import SwiftUI
import ConnectionManager
import CredentialsManager
import TunnelStatus
import UIComponents
import Theme

/// The "Log out of this device?" confirmation and the logout itself, shared by Settings and Your account.
@MainActor
private struct LogoutDialogModifier: ViewModifier {
    @Binding var isPresented: Bool
    let credentialsManager: CredentialsManager
    let connectionManager: ConnectionManager
    let onLoggedOut: () -> Void

    @State private var isLoading = false
    @State private var progressText: String?

    func body(content: Content) -> some View {
        content
            .overlay {
                if isPresented {
                    ActionDialogView(
                        viewModel: ActionDialogViewModel(
                            isDisplayed: $isPresented,
                            configuration: configuration,
                            impactGenerator: .shared,
                            isLoading: $isLoading,
                            loadingTextOverride: $progressText
                        )
                    )
                }
            }
    }

    private var configuration: ActionDialogConfiguration {
        ActionDialogConfiguration(
            systemIconImageName: "rectangle.portrait.and.arrow.right",
            titleLocalizedString: "settings.logoutTitle".localizedString,
            subtitleLocalizedString: "settings.logoutSubtitle".localizedString,
            yesLocalizedString: "settings.logout".localizedString,
            noLocalizedString: "cancel".localizedString,
            isYesDestructive: true,
            yesAction: {
                isLoading = true
                progressText = nil
                Task {
                    await logout()
                    try? await Task.sleep(for: .seconds(0.3))
                    Task { @MainActor in
                        isPresented = false
                        isLoading = false
                        progressText = nil
                        onLoggedOut()
                    }
                }
            },
            loadingText: "settings.loggingOut".localizedString,
            shouldCloseAfterYesAction: false,
            verticalButtonsLayout: true
        )
    }

    private func logout() async {
        await credentialsManager.beginLogout()
        defer { credentialsManager.endLogout() }

        if LogoutTeardownPolicy.needsDisconnectWait(for: connectionManager.currentTunnelStatus) {
            progressText = "disconnecting".localizedString
        }
        await connectionManager.disconnectBeforeLogout()

        progressText = "settings.loggingOut".localizedString
        try? await credentialsManager.removeCredential()
        progressText = nil
    }
}

extension View {
    func logoutDialog(
        isPresented: Binding<Bool>,
        credentialsManager: CredentialsManager,
        connectionManager: ConnectionManager,
        onLoggedOut: @escaping () -> Void
    ) -> some View {
        modifier(
            LogoutDialogModifier(
                isPresented: isPresented,
                credentialsManager: credentialsManager,
                connectionManager: connectionManager,
                onLoggedOut: onLoggedOut
            )
        )
    }
}
