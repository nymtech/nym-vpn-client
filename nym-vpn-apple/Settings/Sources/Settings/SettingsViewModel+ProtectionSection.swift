import SwiftUI
import UIComponents
import Theme

extension SettingsViewModel {
    func protectionSection() -> AppSettingsSection {
        let adBlockViewModel = SettingsListItemViewModel(
            accessory: .toggle(
                isOn: Binding(
                    get: { [appSettings] in appSettings.isAdBlockerEnabled },
                    set: { [connectionManager] newValue in connectionManager.setAdBlocking(newValue) }
                )
            ),
            title: "settings.adblock.title".localizedString,
            subtitle: Self.adBlockSubtitle(isOn: appSettings.isAdBlockerEnabled),
            systemImageName: "exclamationmark.shield",
            action: {}
        )
        appSettings.$isAdBlockerEnabledPublisher
            .dropFirst()
            .receive(on: DispatchQueue.main)
            .sink { isOn in
                adBlockViewModel.subtitle = AttributedString(Self.adBlockSubtitle(isOn: isOn))
            }
            .store(in: &cancellables)

        return AppSettingsSection(
            kind: .protection,
            viewModels: [
                adBlockViewModel,
                SettingsListItemViewModel(
                    accessory: .empty,
                    title: "settings.killswitch.title".localizedString,
                    subtitle: "settings.killswitch.subtitle".localizedString,
                    systemImageName: "power",
                    isHoveredHighlightDisabled: true,
                    action: {}
                ),
                SettingsListItemViewModel(
                    accessory: .arrow,
                    title: "settings.mixnetTuning.title".localizedString,
                    subtitle: "settings.mixnetTuning.subtitle".localizedString,
                    systemImageName: "eye.slash",
                    action: { [weak self] in
                        self?.navigate(to: .mixnetTuning)
                    }
                )
            ]
        )
    }
}

private extension SettingsViewModel {
    nonisolated static func adBlockSubtitle(isOn: Bool) -> String {
        isOn ? "settings.adblock.subtitle.on".localizedString : "settings.adblock.subtitle.off".localizedString
    }
}
