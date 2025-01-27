import SwiftUI
import AppSettings
import CredentialsManager
import ConnectionManager
import ConfigurationManager
#if os(iOS)
import KeyboardManager
#endif
import Theme

final class AddCredentialsViewModel: ObservableObject {
    private let credentialsManager: CredentialsManager
    private let configurationManager: ConfigurationManager
#if os(iOS)
    private let keyboardManager: KeyboardManager
#endif
    private let newToNymVPNTitle = "addCredentials.newToNymVPN".localizedString
    private let createAccountTitle = "addCredentials.createAccount".localizedString

    var signUpLink: String {
        if let link = configurationManager.accountLinks?.signUp, !link.isEmpty {
            return link
        } else {
            return "https://nym.com/account/create"
        }
    }

    let signUpLinkFallback = "https://nym.com/account/create"

    let appSettings: AppSettings
    let loginButtonTitle = "addCredentials.Login.Title".localizedString
    let welcomeTitle = "addCredentials.welcome.Title".localizedString
    let getStartedTitle = "addCredentials.getStarted.Title".localizedString
    let mnemonicSubtitle = "addCredtenials.mnemonic".localizedString
    let credentialsPlaceholderTitle = "addCredentials.placeholder".localizedString
    let scannerIconName = "qrcode.viewfinder"

    @Binding private var path: NavigationPath

    @MainActor @Published var credentialText = "" {
        willSet(newText) {
            guard newText != credentialText else { return }
            error = CredentialsManagerError.noError

            scannerDidScanQRCode()
        }
    }
    @Published var error: Error = CredentialsManagerError.noError {
        didSet {
            Task {
                await configureError()
            }
        }
    }
    @Published var textFieldStrokeColor = NymColor.sysOutlineVariant
    @Published var credentialSubtitleColor = NymColor.sysOnSurface
    @Published var bottomPadding: CGFloat = 12
    @Published var errorMessageTitle = ""
    @MainActor @Published var isScannerDisplayed = false
    @Published var isFocused = true

#if os(iOS)
    init(
        path: Binding<NavigationPath>,
        appSettings: AppSettings = AppSettings.shared,
        credentialsManager: CredentialsManager = CredentialsManager.shared,
        configurationManager: ConfigurationManager = ConfigurationManager.shared,
        keyboardManager: KeyboardManager = KeyboardManager.shared
    ) {
        _path = path
        self.appSettings = appSettings
        self.credentialsManager = credentialsManager
        self.configurationManager = configurationManager
        self.keyboardManager = keyboardManager
    }
#endif
#if os(macOS)
    init(
        path: Binding<NavigationPath>,
        appSettings: AppSettings = AppSettings.shared,
        configurationManager: ConfigurationManager = ConfigurationManager.shared,
        credentialsManager: CredentialsManager = CredentialsManager.shared
    ) {
        _path = path
        self.appSettings = appSettings
        self.configurationManager = configurationManager
        self.credentialsManager = credentialsManager
    }
#endif

    func createAnAccountAttributedString() -> AttributedString? {
        try? AttributedString(markdown: "\(newToNymVPNTitle) [\(createAccountTitle)](\(signUpLink))")
    }

    @MainActor func importCredentials() {
        error = CredentialsManagerError.noError
        let trimmedCredential = credentialText.trimmingCharacters(in: .whitespacesAndNewlines)

        Task {
            do {
                try await credentialsManager.add(credential: trimmedCredential)
                credentialsDidAdd()
            } catch let newError {
                Task { @MainActor in
                    credentialText = trimmedCredential
                    error = CredentialsManagerError.generalError(String(describing: newError.localizedDescription))
                }
            }
        }
    }
}

// MARK: - Navigation -
extension AddCredentialsViewModel {
    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }

    func navigateHome() {
        path = .init()
    }
}

// MARK: - Private -
extension AddCredentialsViewModel {
    @MainActor func configureError() {
        let error = error as? CredentialsManagerError

        textFieldStrokeColor = error == .noError ? NymColor.sysOutlineVariant : NymColor.sysError
        credentialSubtitleColor = error == .noError ? NymColor.sysOnSurface : NymColor.sysError
        bottomPadding = error != .noError ? 4 : 12

        errorMessageTitle = (error == .noError ? "" : error?.localizedTitle)
        ?? (CredentialsManagerError.generalError("").localizedTitle ?? "Error".localizedString)
    }

    @MainActor func credentialsDidAdd() {
        credentialText = ""
        navigateHome()
    }

    @MainActor func scannerDidScanQRCode() {
#if os(iOS)
        if isScannerDisplayed {
            isFocused = false
            isScannerDisplayed = false
            keyboardManager.hideKeyboard()
            importCredentials()
        }
#endif
    }
}
