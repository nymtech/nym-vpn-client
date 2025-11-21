import SwiftUI
import AppVersionProvider
import ConfigurationManager
import Device
import Theme

@MainActor public struct SettingsListViewModel<Kind: SettingsSectionKind> {
    let sections: [SettingsSection<Kind>]

    public init(sections: [SettingsSection<Kind>]) {
        self.sections = sections
    }
}
