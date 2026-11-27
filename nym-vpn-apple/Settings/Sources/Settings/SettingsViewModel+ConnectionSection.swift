import SwiftUI
import UIComponents
import Theme

extension SettingsViewModel {
    func connectionSection() -> AppSettingsSection {
        var viewModels = [SettingsListItemViewModel]()
#if os(macOS)
        viewModels.append(
            SettingsListItemViewModel(
                accessory: .toggle(
                    isOn: Binding(
                        get: { [appSettings] in appSettings.isIPv6TrafficEnabled },
                        set: { [connectionManager] newValue in connectionManager.setIPv6TrafficEnabled(newValue) }
                    )
                ),
                title: "settings.ipv6.title".localizedString,
                subtitle: "settings.ipv6.subtitle".localizedString,
                imageName: "powerplug.portrait",
                action: {}
            )
        )
#endif
        viewModels.append(
            contentsOf: [
                SettingsListItemViewModel(
                    accessory: .toggle(
                        isOn: Binding(
                            get: { [appSettings] in appSettings.isLanBypassEnabled },
                            set: { [connectionManager] newValue in connectionManager.setLanBypassEnabled(newValue) }
                        )
                    ),
                    title: "settings.lanBypass.title".localizedString,
                    subtitle: "settings.lanBypass.subtitle".localizedString,
                    imageName: "lan",
                    action: {}
                ),
                SettingsListItemViewModel(
                    accessory: .arrow,
                    title: "settings.censorship.title".localizedString,
                    imageName: "domain",
                    action: { [weak self] in
                        Task { @MainActor in
                            self?.navigate(to: .censorship)
                        }
                    }
                ),
                SettingsListItemViewModel(
                    accessory: .arrow,
                    title: "settings.profiles.title".localizedString,
                    subtitle: "settings.profiles.subtitle".localizedString,
                    imageName: "profiles",
                    action: { [weak self] in
                        self?.navigate(to: .profiles)
                    }
                ),
                SettingsListItemViewModel(
                    accessory: .arrow,
                    title: "settings.dns.title".localizedString,
                    imageName: "dns",
                    action: { [weak self] in
                        Task { @MainActor in
                            self?.navigate(to: .dns)
                        }
                    }
                )
            ]
        )
        return AppSettingsSection(kind: .connection, viewModels: viewModels)
    }
}
