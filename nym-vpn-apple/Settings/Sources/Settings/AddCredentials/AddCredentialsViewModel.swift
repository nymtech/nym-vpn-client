import SwiftUI
import AppSettings
import CredentialsManager
import MixnetLibrary
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

    @Published var credentialText = ""
    @Published var error: Error = CredentialsManagerError.noError {
        didSet {
            configureError()
        }
    }
    @Published var textFieldStrokeColor = NymColor.sysOutlineVariant
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
                appSettings.isCredentialImported = true
                navigateHome()
            } catch let newError {
                Task { @MainActor in
                    error = newError
                }
            }
        }
    }
}

// MARK: - Navigation -
extension AddCredentialsViewModel {
    func navigateBack() {
        path.removeLast()
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
            bottomPadding = error != .noError ? 4 : 12

            errorMessageTitle = (error == .noError ? "" : error?.localizedTitle)
            ?? CredentialsManagerError.unknownError.localizedDescription
        }
    }
}
