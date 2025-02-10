import Foundation
#if os(macOS)
import HelperInstall
#endif

public enum SettingLink: Hashable, Identifiable {
    case addCredentials
    case theme
    case logs
    case support
    case legal
    case acknowledgments
    case licence(details: LicenceDetails)
    case santasMenu
#if os(macOS)
    case installHelper(afterInstallAction: HelperAfterInstallAction)
#endif

    public var id: String {
        String(describing: self)
    }
}
