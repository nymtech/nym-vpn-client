import SwiftUI
import AppSettings
import Constants
import CredentialsManager
import ConnectionManager
import ConfigurationManager
#if os(iOS)
import KeyboardManager
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
    let scannerIconName = "qrcode.viewfinder"
    let navigationSource: AddCredentialsNavigationSource
    let createAccounAppLink = "app://createAccount"

    var signUpLink: String {
        if let link = configurationManager.accountLinks?.signUp, !link.isEmpty {
            link
        } else {
            Constants.pricingURL.rawValue
        }
    }

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
    @Published var textFieldStrokeColor = NymColor.gray2
    @Published var credentialSubtitleColor = NymColor.primary
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
                try await credentialsManager.add(credential: trimmedCredential)
                error = CredentialsManagerError.noError
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
        switch navigationSource {
        case .onboarding:
            path = .init([HomeLink.onboarding])
        case .createAccountWelcome:
//            path = .init([HomeLink.settings])
//            // TODO: check if true
//            path.append(SettingLink.createAccountWelcome(navigationSource: .home))
            if !path.isEmpty { path.removeLast() }
        case .settings:
            if !path.isEmpty { path.removeLast() }
        }
    }

    func navigateToCreateAccount() {
        path = NavigationPath([HomeLink.settings])
        path.append(SettingLink.createAccountWelcome(navigationSource: .addCredential))
    }

    func navigateHomeOrTechnicalOptIn() {
        if appSettings.welcomeScreenDidDisplay {
            path = .init()
        } else {
            path = .init([HomeLink.technicalOptIns])
        }
    }
}

// MARK: - Private -
extension AddCredentialsViewModel {
    @MainActor func configureError() {
        let error = error as? CredentialsManagerError

        textFieldStrokeColor = error == .noError ? NymColor.gray2 : NymColor.error
        credentialSubtitleColor = error == .noError ? NymColor.primary : NymColor.error
        bottomPadding = error != .noError ? 4 : 8

        errorMessageTitle = (error == .noError ? "" : error?.localizedTitle)
        ?? (CredentialsManagerError.generalError("").localizedTitle ?? "Error".localizedString)
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
