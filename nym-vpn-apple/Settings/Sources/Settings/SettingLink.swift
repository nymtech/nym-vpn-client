import Foundation

public enum SettingLink: Hashable, Identifiable {
    case addCredentials
    case theme
    case logs
    case feedback
    case support
    case legal
    case acknowledgments
    case licence(details: LicenceDetails)
    case santasMenu

    public var id: String {
        String(describing: self)
    }
}
