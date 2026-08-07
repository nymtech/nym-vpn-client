import SwiftUI
import AppSettings
import CredentialsManager
import ConnectionManager
import ConfigurationManager
#if os(iOS)
import ErrorHandler
import KeyboardManager
import NymVPNLib
#endif
import Routes
import Theme

@MainActor final class AddCredentialsViewModel: ObservableObject {
    private let credentialsManager: CredentialsManager
    private let configurationManager: ConfigurationManager
#if os(iOS)
    private let keyboardManager: KeyboardManager
#endif
    private let newToNymVPNTitle = "addCredentials.newToNymVPN".localizedString
    private let createAccountTitle = "createAccount".localizedString


    @Binding private var path: NavigationPath

    let appSettings: AppSettings
    let navigationSource: AddCredentialsNavigationSource
    let createAccounAppLink = "app://createAccount"

    @MainActor @Published var credentialText = "" {
        willSet(newText) {
            guard newText != credentialText else { return }
            error = CredentialsManagerError.noError

            scannerDidScanQRCode()
        }
    }
    @Published var error: Error = CredentialsManagerError.noError {
        didSet {
            configureError()
        }
    }
    @Published var textFieldStrokeColor = Color.Nym.textSecondary
    @Published var credentialSubtitleColor = Color.Nym.textPrimary
    @Published var bottomPadding: CGFloat = 8
    @Published var errorMessageTitle = ""
    @MainActor @Published var isScannerDisplayed = false
    @Published var isFocused = true

#if os(iOS)
    init(
        path: Binding<NavigationPath>,
        appSettings: AppSettings,
        credentialsManager: CredentialsManager,
        configurationManager: ConfigurationManager,
        keyboardManager: KeyboardManager,
        navigationSource: AddCredentialsNavigationSource
    ) {
        _path = path
        self.appSettings = appSettings
        self.credentialsManager = credentialsManager
        self.configurationManager = configurationManager
        self.keyboardManager = keyboardManager
        self.navigationSource = navigationSource
    }
#elseif os(macOS)
    init(
        path: Binding<NavigationPath>,
        appSettings: AppSettings,
        configurationManager: ConfigurationManager,
        credentialsManager: CredentialsManager,
        navigationSource: AddCredentialsNavigationSource
    ) {
        _path = path
        self.appSettings = appSettings
        self.configurationManager = configurationManager
        self.credentialsManager = credentialsManager
        self.navigationSource = navigationSource
    }
#endif

    func createAnAccountAttributedString() -> AttributedString? {
        try? AttributedString(markdown: "\(newToNymVPNTitle) [\(createAccountTitle)](\(createAccounAppLink))")
    }

    @MainActor func importCredentials() {
        let trimmedCredential = credentialText.trimmingCharacters(in: .whitespacesAndNewlines)

        Task {
            do {
#if os(iOS)
                try await credentialsManager.performAccountRegistration(loginCredential: trimmedCredential)
#elseif os(macOS)
                // add → grpc storeAccount persists the mnemonic on the daemon; registerAccount() is iOS-only.
                try await credentialsManager.add(credential: trimmedCredential)
                await credentialsManager.updateAccountSummary(force: true, untilActive: false)
#endif
                error = CredentialsManagerError.noError
                credentialsDidAdd()
            } catch let newError {
                Task { @MainActor in
                    credentialText = trimmedCredential
#if os(iOS)
                    if let reason = newError as? VPNErrorReason {
                        error = CredentialsManagerError.generalError(reason.localizedDescription)
                    } else if let vpnError = newError as? VpnError {
                        error = CredentialsManagerError.generalError(
                            VPNErrorReason(with: vpnError).localizedDescription
                        )
                    } else {
                        error = CredentialsManagerError.generalError(newError.localizedDescription)
                    }
#elseif os(macOS)
                    error = CredentialsManagerError.generalError(newError.localizedDescription)
#endif
                }
            }
        }
    }
}

// MARK: - Navigation -
extension AddCredentialsViewModel {
    func navigateBack() {
        switch navigationSource {
        case .onboarding:
            path = .init()
        case .accountWelcome:
            if !path.isEmpty { path.removeLast() }
        case .settings:
            if !path.isEmpty { path.removeLast() }
        }
    }

    func navigateToCreateAccount() {
        path = NavigationPath([HomeLink.settings])
        path.append(SettingLink.accountWelcome(type: .createAccount, navigationSource: .addCredential))
    }

    func navigateHomeOrTechnicalOptIn() {
        path = .init()
    }
}

// MARK: - Private -
extension AddCredentialsViewModel {
    @MainActor func configureError() {
        let error = error as? CredentialsManagerError

        textFieldStrokeColor = error == .noError ? Color.Nym.textSecondary : Color.Nym.error
        credentialSubtitleColor = error == .noError ? Color.Nym.textPrimary : Color.Nym.error
        bottomPadding = error != .noError ? 4 : 8

        errorMessageTitle = (error == .noError ? "" : error?.localizedTitle)
        ?? (CredentialsManagerError.generalError("").localizedTitle ?? "error".localizedString)
    }

    @MainActor func credentialsDidAdd() {
        credentialText = ""
        navigateHomeOrTechnicalOptIn()
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
