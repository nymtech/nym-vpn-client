import Theme
import UIComponents

/// Settings sections in display order; a nil `title` means no header.
public enum AppSettingsSectionKind: SettingsSectionKind {
    case account
    case protection
    case connection
    case routing
    case app
    case helpAndData
    case footer

    var title: String? {
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
        case .footer:
            nil
        }
    }
}
