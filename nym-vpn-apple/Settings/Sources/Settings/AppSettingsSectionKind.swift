import Theme
import UIComponents

public enum AppSettingsSectionKind: SettingsSectionKind {
    case account
    case protection
    case connection
    case routing
    case app
    case helpAndData

    var title: String {
        switch self {
        case .account:
            "settings.section.account".localizedString
        case .protection:
            "settings.section.protection".localizedString
        case .connection:
            "settings.section.connection".localizedString
        case .routing:
            "settings.section.routing".localizedString
        case .app:
            "settings.section.app".localizedString
        case .helpAndData:
            "settings.section.helpAndData".localizedString
        }
    }
}
