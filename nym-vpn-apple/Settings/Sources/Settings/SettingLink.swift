import Foundation

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
    case censorship
#if os(macOS)
    case appMode
    case daemonEnable
#endif

    public var id: String {
        String(describing: self)
    }
}
