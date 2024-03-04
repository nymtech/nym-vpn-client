import Foundation

enum SettingsLink: Hashable, Identifiable {
    case theme
    case support

    var id: String {
        String(describing: self)
    }
}
