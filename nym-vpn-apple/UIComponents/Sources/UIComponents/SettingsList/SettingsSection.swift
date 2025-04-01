import Foundation

public enum SettingsSection: Hashable {
    case account(viewModels: [SettingsListItemViewModel])
    case connection(viewModels: [SettingsListItemViewModel])
    case theme(viewModels: [SettingsListItemViewModel])
    case logs(viewModels: [SettingsListItemViewModel])
    case feedback(viewModels: [SettingsListItemViewModel])
    case killSwitch(viewModels: [SettingsListItemViewModel])
    case legal(viewModels: [SettingsListItemViewModel])
    case logout(viewModels: [SettingsListItemViewModel])

    var settingsViewModels: [SettingsListItemViewModel] {
        switch self {
        case let .account(viewModels),
            let .connection(viewModels),
            let .theme(viewModels),
            let .logs(viewModels),
            let .feedback(viewModels),
            let .killSwitch(viewModels),
            let .legal(viewModels),
            let .logout(viewModels):
            return viewModels
        }
    }
}
