import Foundation

public enum SettingsLink: Hashable, Identifiable {
    case addCredentials
    case theme
    case feedback
    case support
    case legal

    public var id: String {
        String(describing: self)
    }
}
