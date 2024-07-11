import SwiftUI
import AppSettings
import CredentialsManager
import Theme

final class AddCredentialsViewModel: ObservableObject {
    let addCredentialButtonTitle = "addCredentials.addCredential.Title".localizedString
    let welcomeTitle = "addCredentials.welcome.Title".localizedString
    let getStartedTitle = "addCredentials.getStarted.Title".localizedString
    let getStartedSubtitle = "addCredentialsGetStarted.Subtitle".localizedString
    let credentialSubtitle = "addCredtenials.credential".localizedString
    let credentialsPlaceholderTitle = "".localizedString
    let logoImageName = "addCredentialsLogo"
    let appSettings: AppSettings
    let credentialsManager: CredentialsManager

    @Binding var path: NavigationPath

    @Published var credentialText = "" {
        willSet(newText) {
            guard newText != credentialText else { return }
            error = CredentialsManagerError.noError
        }
    }
    @Published var error: Error = CredentialsManagerError.noError {
        didSet {
            configureError()
        }
    }
    @Published var textFieldStrokeColor = NymColor.sysOutlineVariant
    @Published var credentialSubtitleColor = NymColor.sysOnSurface
    @Published var bottomPadding: CGFloat = 12
    @Published var errorMessageTitle = ""

    init(
        path: Binding<NavigationPath>,
        appSettings: AppSettings = AppSettings.shared,
        credentialsManager: CredentialsManager = CredentialsManager.shared
    ) {
        _path = path
        self.appSettings = appSettings
        self.credentialsManager = credentialsManager
    }

    func importCredentials() {
        Task { @MainActor in
            error = CredentialsManagerError.noError
        }
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
    func configureError() {
        Task { @MainActor in
            let error = error as? CredentialsManagerError

            textFieldStrokeColor = error == .noError ? NymColor.sysOutlineVariant : NymColor.sysError
            credentialSubtitleColor = error == .noError ? NymColor.sysOnSurface : NymColor.sysError
            bottomPadding = error != .noError ? 4 : 12

            errorMessageTitle = (error == .noError ? "" : error?.localizedTitle)
            ?? (CredentialsManagerError.generalError("").localizedTitle ?? "Error".localizedString)
        }
    }

    func credentialsDidAdd() {
        Task { @MainActor in
            appSettings.isCredentialImported = true
            credentialText = ""
            navigateHome()
        }
    }
}
