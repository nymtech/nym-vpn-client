import Foundation

public enum SettingLink: Hashable, Identifiable {
    case accountAndDevices
    case addCredentials(navigationSource: AddCredentialsNavigationSource)
    case accountWelcome(type: AccountWelcomeType, navigationSource: AccountWelcomeNavigationSource)
    case generatePassphrase(displayPurchaseView: Bool)
    case processingAccount
    case passphrase
    case appearance
    case displayTheme
#if os(iOS)
    case appIcon
#endif
    case logs
    case support
    case legal
    case systemStatus
    case acknowledgments
    case licence(details: LicenceDetails)
#if SANTA
    case santasMenu
#endif
    case privacyAndData
    case dns
    case mixnetTuning
    case censorship
#if os(macOS)
    case proxy
    case appMode
    case daemonEnable
    case splitTunnel
    case diagnosticTool
#endif

    public var id: String {
        String(describing: self)
    }
}
