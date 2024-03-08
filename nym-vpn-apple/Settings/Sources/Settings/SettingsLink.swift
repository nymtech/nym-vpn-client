import Foundation

enum SettingsLink: Hashable, Identifiable {
    case theme
    case feedback
    case support
    case legal
    case survey

    var id: String {
        String(describing: self)
    }
}
