import Foundation

public enum HopType: Codable, Hashable, Sendable {
    case entry
    case exit

    public var hopLocalizedTitle: String {
        switch self {
        case .entry:
            "home.entryHop".localizedString
        case .exit:
            "home.exitHop".localizedString
        }
    }
}
