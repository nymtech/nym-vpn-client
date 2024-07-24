import Foundation

public enum SettingsLink: Hashable, Identifiable {
    case addCredentials
    case theme
    case logs
    case feedback
    case support
    case legal

    public var id: String {
        String(describing: self)
    }
}
