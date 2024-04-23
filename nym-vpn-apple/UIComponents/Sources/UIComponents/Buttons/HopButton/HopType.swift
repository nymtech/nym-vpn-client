import Foundation

public enum HopType {
    case entry
    case exit

    public var selectHopLocalizedTitle: String {
        switch self {
        case .entry:
            "firstHopSelection".localizedString
        case .exit:
            "lastHopSelection".localizedString
        }
    }

    public var hopLocalizedTitle: String {
        switch self {
        case .entry:
            "home.entryHop".localizedString
        case .exit:
            "home.exitHop".localizedString
        }
    }
}
