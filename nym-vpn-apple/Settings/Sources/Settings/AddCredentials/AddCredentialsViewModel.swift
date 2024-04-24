import SwiftUI
import Theme

final class AddCredentialsViewModel: ObservableObject {
    let addCredentialButtonTitle = "addCredentials.addCredential.Title".localizedString
    let welcomeTitle = "addCredentials.welcome.Title".localizedString
    let getStartedTitle = "addCredentials.getStarted.Title".localizedString
    let getStartedSubtitle = "addCredentialsGetStarted.Subtitle".localizedString
    let credentialSubtitle = "addCredtenials.credential".localizedString
    let credentialsPlaceholderTitle = "".localizedString
    let logoImageName = "addCredentialsLogo"

    var textFieldStrokeColor: Color {
        error == .invalidCredential ? NymColor.sysError : NymColor.sysOutlineVariant
    }

    @Binding var path: NavigationPath

    @Published var credentialText = ""
    @Published var error: AddCredentialsError = .noError

    init(path: Binding<NavigationPath>) {
        _path = path
    }
}

// MARK: - Navigation -
extension AddCredentialsViewModel {
    func navigateBack() {
        path.removeLast()
    }
}
