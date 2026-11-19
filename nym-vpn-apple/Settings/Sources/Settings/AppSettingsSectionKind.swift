import Theme
import UIComponents

public enum AppSettingsSectionKind: SettingsSectionKind {
    case account
    case redeemCode
    case protection
    case connection
    case routing
    case mixnet
    case app
    case helpAndData

    var title: String {
        switch self {
        case .account, .redeemCode:
            "settings.section.account".localizedString
        case .protection:
            "settings.section.protection".localizedString
        case .connection:
            "settings.section.connection".localizedString
        case .routing:
            "settings.section.routing".localizedString
        case .mixnet:
            "settings.section.mixnet".localizedString
        case .app:
            "settings.section.app".localizedString
        case .helpAndData:
            "settings.section.helpAndData".localizedString
        }
    }
}
