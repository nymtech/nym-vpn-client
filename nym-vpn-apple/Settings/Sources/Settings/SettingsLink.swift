import Foundation

enum SettingsLink: Hashable, Identifiable {
    case theme
    case support
    case legal

    var id: String {
        String(describing: self)
    }
}
