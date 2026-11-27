import SwiftUI
#if os(iOS)
import LocalAuthentication
import BiometricAuthenticator
import SnackbarManager
#endif
import CredentialsManager
import Theme
import UIComponents

extension AccountAndDevicesView {
    func pairDeviceButton() -> some View {
        NymButton("pairDevice.button".localizedString, style: .surface) {
            impactGenerator.softImpact()
            startPairDevice()
        }
    }

    func startPairDevice() {
#if os(iOS)
        Task {
            do {
                try await BiometricAuthenticator.authenticate(reason: "pairDevice.authReason".localizedString)
                presentPairDevice()
            } catch let error as LAError where [.userCancel, .systemCancel, .appCancel].contains(error.code) {
                return
            } catch {
                SnackbarManager.shared.enqueue(
                    SnackbarItem(style: .negative, title: "pairDevice.authError".localizedString)
                )
            }
        }
#elseif os(macOS)
        // The daemon asks the user to confirm when the code starts.
        presentPairDevice()
#endif
    }

    func presentPairDevice() {
        isPairDevicePresented = true
        // Started here, not on appear, so a sheet redraw can't restart the session.
        Task { await PairingManager.shared.startShowingCode() }
    }
}
