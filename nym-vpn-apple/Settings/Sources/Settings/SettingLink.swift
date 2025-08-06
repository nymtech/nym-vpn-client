import Foundation
#if os(macOS)
import HelperInstall
#endif

public enum SettingLink: Hashable, Identifiable {
    case addCredentials
    case createAccountWelcome
    case createAccount
    case createAccountSuccess
    case planPurchaseSuccess
    case appearance
    case displayTheme
    case logs
    case support
    case legal
    case acknowledgments
    case licence(details: LicenceDetails)
    case santasMenu
    case privacyAndData
#if os(macOS)
    case installHelper(afterInstallAction: HelperAfterInstallAction)
    case appMode
#endif

    public var id: String {
        String(describing: self)
    }
}
