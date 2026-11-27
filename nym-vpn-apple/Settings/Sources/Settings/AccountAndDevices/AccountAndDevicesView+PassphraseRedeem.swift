import SwiftUI
import CredentialsManager
import UIComponents

extension AccountAndDevicesView {
    /// Passphrase is iOS only; Redeem a code shows on both platforms.
    func passphraseAndRedeemSection() -> some View {
        VStack(spacing: 0) {
#if os(iOS)
            SettingsListItem(
                viewModel: SettingsListItemViewModel(
                    accessory: .arrow,
                    title: "settings.passphrase".localizedString,
                    imageName: "key",
                    position: SettingsListItemPosition(isFirst: true, isLast: false),
                    action: {
                        impactGenerator.softImpact()
                        path.append(SettingLink.passphrase)
                    }
                )
            )
#endif
            SettingsListItem(
                viewModel: SettingsListItemViewModel(
                    accessory: .arrow,
                    title: "settings.redeemCode".localizedString,
                    systemImageName: "ticket",
                    position: SettingsListItemPosition(isFirst: redeemIsFirst, isLast: true),
                    action: {
                        impactGenerator.softImpact()
                        isCodeEntryPresented = true
                    }
                )
            )
        }
    }

    func handleCodeEntry(_ outcome: FreepassOrPairingOutcome) {
        isCodeEntryPresented = false
        if outcome != .cancelled {
            path = NavigationPath()
        }
        Task { [credentialsManager, onSessionEvent] in
            await FreepassOrPairingOutcomeHandler.handle(
                outcome,
                credentialsManager: credentialsManager,
                sendSessionEvent: { onSessionEvent($0) }
            )
        }
    }
}

private extension AccountAndDevicesView {
    var redeemIsFirst: Bool {
#if os(iOS)
        false
#else
        true
#endif
    }
}
