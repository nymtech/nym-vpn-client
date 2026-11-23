#if os(iOS)
import LocalAuthentication
import SwiftUI
import BiometricAuthenticator
import SnackbarManager
import Theme
import UIComponents

extension AccountAndDevicesView {
    func pairDeviceSection() -> some View {
        SettingsListItem(
            viewModel: SettingsListItemViewModel(
                accessory: .arrow,
                title: "pairDevice.button".localizedString,
                systemImageName: "qrcode",
                position: SettingsListItemPosition(isFirst: true, isLast: true),
                action: {
                    impactGenerator.softImpact()
                    startPairDevice()
                }
            )
        )
    }

    func startPairDevice() {
        Task {
            do {
                try await BiometricAuthenticator.authenticate(reason: "pairDevice.authReason".localizedString)
                isPairDevicePresented = true
            } catch let error as LAError where [.userCancel, .systemCancel, .appCancel].contains(error.code) {
                return
            } catch {
                SnackbarManager.shared.enqueue(
                    SnackbarItem(style: .negative, title: "pairDevice.authError".localizedString)
                )
            }
        }
    }
}
#endif
