import Foundation

public enum SettingsLink: Hashable, Identifiable {
    case addCredentials
    case theme
    case logs
    case feedback
    case support
    case legal
    case acknowledgments
    case licence(details: LicenceDetails)

    public var id: String {
        String(describing: self)
    }
}
