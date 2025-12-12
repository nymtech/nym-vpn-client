import Foundation

public enum SettingLink: Hashable, Identifiable {
    case accountAndDevices
    case addCredentials
    case createAccountWelcome
    case generatePassphrase
    case planPurchase(shouldDisplayBackButton: Bool)
    case processingAccount
    case passphrase
    case appearance
    case displayTheme
    case logs
    case support
    case legal
    case systemStatus
    case acknowledgments
    case licence(details: LicenceDetails)
    case santasMenu
    case privacyAndData
    case dns
    case censorship
#if os(macOS)
    case proxy
    case appMode
    case daemonEnable
#endif

    public var id: String {
        String(describing: self)
    }
}
