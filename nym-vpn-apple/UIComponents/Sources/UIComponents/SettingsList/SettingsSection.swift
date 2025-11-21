import Foundation

public protocol SettingsSectionKind: Hashable {}

public struct SettingsSection<Kind: SettingsSectionKind>: Hashable {
    public let kind: Kind
    public let viewModels: [SettingsListItemViewModel]

    public init(kind: Kind, viewModels: [SettingsListItemViewModel]) {
        self.kind = kind
        self.viewModels = viewModels
    }
}
