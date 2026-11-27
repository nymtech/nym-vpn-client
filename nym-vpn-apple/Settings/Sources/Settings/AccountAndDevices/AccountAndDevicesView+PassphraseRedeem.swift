#if os(iOS)
import SwiftUI
import CredentialsManager
import UIComponents

extension AccountAndDevicesView {
    func passphraseAndRedeemSection() -> some View {
        VStack(spacing: 0) {
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
            SettingsListItem(
                viewModel: SettingsListItemViewModel(
                    accessory: .arrow,
                    title: "settings.redeemCode".localizedString,
                    systemImageName: "ticket",
                    position: SettingsListItemPosition(isFirst: false, isLast: true),
                    action: {
                        impactGenerator.softImpact()
                        isCodeEntryPresented = true
                    }
                )
            )
        }
    }

    func handleCodeEntry(_ outcome: CodeEntryOutcome) {
        isCodeEntryPresented = false
        if outcome != .cancelled {
            path = NavigationPath()
        }
        Task { [credentialsManager, onSessionEvent] in
            await CodeEntryOutcomeHandler.handle(
                outcome,
                credentialsManager: credentialsManager,
                sendSessionEvent: { onSessionEvent($0) }
            )
        }
    }
}
#endif
