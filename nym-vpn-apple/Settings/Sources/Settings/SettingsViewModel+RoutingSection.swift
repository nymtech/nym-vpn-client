import SwiftUI
import UIComponents
import Theme

extension SettingsViewModel {
    /// macOS only; nil on iOS, which has none of these rows.
    func routingSection() -> AppSettingsSection? {
#if os(macOS)
        AppSettingsSection(
            kind: .routing,
            viewModels: [
                SettingsListItemViewModel(
                    accessory: .arrow,
                    title: "settings.splitTunnel".localizedString,
                    imageName: "arrow.trianglehead.branch",
                    action: { [weak self] in
                        self?.navigate(to: .splitTunnel)
                    }
                ),
                SettingsListItemViewModel(
                    accessory: .arrow,
                    title: "settings.geoExclusion".localizedString,
                    subtitle: "settings.geoExclusion.subtitle".localizedString,
                    imageName: "pin",
                    badge: "general.beta".localizedString,
                    action: { [weak self] in
                        self?.navigate(to: .geoExclusion)
                    }
                ),
                SettingsListItemViewModel(
                    accessory: .arrow,
                    title: "settings.proxy.title".localizedString,
                    subtitle: "settings.proxy.subtitle".localizedString,
                    imageName: "proxy",
                    badge: "general.beta".localizedString,
                    action: { [weak self] in
                        Task { @MainActor in
                            self?.navigate(to: .proxy)
                        }
                    }
                )
            ]
        )
#else
        nil
#endif
    }
}
