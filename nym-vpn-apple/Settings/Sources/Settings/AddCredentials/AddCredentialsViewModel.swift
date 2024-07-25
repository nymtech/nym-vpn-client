import SwiftUI
import AppSettings
import CredentialsManager
#if os(iOS)
import KeyboardManager
#endif
import Theme

final class AddCredentialsViewModel: ObservableObject {
    private let credentialsManager: CredentialsManager
#if os(iOS)
    private let keyboardManager: KeyboardManager
#endif

    let appSettings: AppSettings
    let addCredentialButtonTitle = "addCredentials.addCredential.Title".localizedString
    let welcomeTitle = "addCredentials.welcome.Title".localizedString
    let getStartedTitle = "addCredentials.getStarted.Title".localizedString
    let getStartedSubtitle = "addCredentialsGetStarted.Subtitle".localizedString
    let credentialSubtitle = "addCredtenials.credential".localizedString
    let credentialsPlaceholderTitle = "".localizedString
    let logoImageName = "addCredentialsLogo"
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
        keyboardManager: KeyboardManager = KeyboardManager.shared
    ) {
        _path = path
        self.appSettings = appSettings
        self.credentialsManager = credentialsManager
        self.keyboardManager = keyboardManager
    }
#endif
#if os(macOS)
    init(
        path: Binding<NavigationPath>,
        appSettings: AppSettings = AppSettings.shared,
        credentialsManager: CredentialsManager = CredentialsManager.shared
    ) {
        _path = path
        self.appSettings = appSettings
        self.credentialsManager = credentialsManager
    }
#endif

    @MainActor func importCredentials() {
        error = CredentialsManagerError.noError

        Task {
            do {
                try credentialsManager.add(credential: credentialText)
                credentialsDidAdd()
            } catch let newError {
                Task { @MainActor in
                    error = CredentialsManagerError.generalError(String(describing: newError))
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
        appSettings.isCredentialImported = true
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
