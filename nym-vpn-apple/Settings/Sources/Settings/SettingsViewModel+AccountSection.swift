import SwiftUI
import ConnectionTypes
import CredentialsManager
import UIComponents
import Theme

extension SettingsViewModel {
    func accountSection() -> AppSettingsSection {
        var viewModels = [SettingsListItemViewModel]()
        if appSettings.isCredentialImported {
            viewModels.append(yourAccountItem())
#if os(iOS)
            viewModels.append(
                SettingsListItemViewModel(
                    accessory: .arrow,
                    title: "settings.passphrase".localizedString,
                    imageName: "key",
                    action: { [weak self] in
                        Task { @MainActor in
                            self?.navigate(to: .passphrase)
                        }
                    }
                )
            )
#endif
        }
        // Redeeming without an account creates one, so it shows either way.
        viewModels.append(
            SettingsListItemViewModel(
                accessory: .arrow,
                title: "settings.redeemCode".localizedString,
                systemImageName: "ticket",
                action: { [weak self] in
                    Task { @MainActor in
                        self?.impactGenerator.softImpact()
                        self?.isCodeEntryPresented = true
                    }
                }
            )
        )
        return AppSettingsSection(kind: .account, viewModels: viewModels)
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
                sendSessionEvent: { onSessionEvent?($0) }
            )
        }
    }
}

private extension SettingsViewModel {
    func yourAccountItem() -> SettingsListItemViewModel {
        let subtitle: AttributedString
        if let accountSummary = credentialsManager.accountSummary {
            if let planText = accountSummary.planValidUntilAttributedString {
                if accountSummary.isActive,
                   isAutoRenewEnabled(accountSummary: accountSummary),
                   !accountSummary.isExpiringSoon,
                   !accountSummary.isExpiringWarning {
                    var second = AttributedString("* \("autoRenews".localizedString)")
                    second.foregroundColor = Color.Nym.textSecondary
                    subtitle = planText + AttributedString("\n") + second
                } else {
                    subtitle = planText
                }
            } else if accountSummary.subscription?.status == .pending {
                var confirmingPayment = AttributedString("confirmingPayment".localizedString)
                confirmingPayment.foregroundColor = Color.Nym.error
                subtitle = confirmingPayment
            } else {
                var first = AttributedString("noActivePlan".localizedString)
                first.foregroundColor = Color.Nym.error
                var second = AttributedString("\n\("purchasePlan.chooseMyPlan".localizedString)")
                second.foregroundColor = Color.Nym.primary
                subtitle = first + second
            }
        } else {
            subtitle = AttributedString("requestingZkNyms".localizedString)
        }

        return SettingsListItemViewModel(
            accessory: .arrow,
            title: "settings.yourAccount".localizedString,
            attributtedSubtitle: subtitle,
            systemImageName: "person.crop.circle",
            action: { [weak self] in
                Task { @MainActor in
                    self?.navigate(to: .accountAndDevices)
                }
            }
        )
    }

    func isAutoRenewEnabled(accountSummary: AccountSummary) -> Bool {
#if os(iOS)
        purchasesManager.isAutoRenewEnabled || accountSummary.isAutoRenewEnabled
#elseif os(macOS)
        accountSummary.isAutoRenewEnabled
#endif
    }
}
