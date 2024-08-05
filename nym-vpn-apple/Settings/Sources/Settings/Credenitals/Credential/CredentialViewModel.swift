import SwiftUI
import CredentialsManager

final class CredentialViewModel: ObservableObject {
    private let credentialsManager: CredentialsManager

    @Binding private var path: NavigationPath

    let title = "addCredentials.Credential".localizedString
    let soonToExpireLocalizedString = "addCredentials.Credential.soonToExpire".localizedString
    let addNewCredentialLocalizedString = "addCredentials.Credential.addNewCredential".localizedString

    var timeUsed: Double {
        let duration = Double(credentialsDuration())
        let daysUsed = duration - Double(credentialDaysLeft())
        let value = (daysUsed / duration)
        let roundedValue = round(value * 100) / 100.0
        return roundedValue
    }

    var displayExtendSection: Bool {
        !credentialsManager.isValidCredentialImported
    }

    init(
        path: Binding<NavigationPath>,
        credentialsManager: CredentialsManager = CredentialsManager.shared
    ) {
        _path = path
        self.credentialsManager = credentialsManager
    }

    func daysLeftLocalizedString() -> String {
        let outOfString = "addCredentials.credential.outOf".localizedString
        let daysLeftString = "addCredentials.credential.daysLeft".localizedString
        return "\(String(credentialDaysLeft())) \(outOfString) \(String(credentialsDuration())) \(daysLeftString)"
    }
}

// MARK: - Navigation -
extension CredentialViewModel {
    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }

    func navigateToAddCredential() {
        path.append(SettingsLink.addCredentials)
    }
}

// MARK: - Date calculations -
private extension CredentialViewModel {
    func credentialDaysLeft() -> Int {
        guard let expiryDate = credentialsManager.expiryDate else { return 0 }
        let difference = Calendar.current.dateComponents([.day], from: Date(), to: expiryDate)
        return difference.day ?? 0
    }

    func credentialsDuration() -> Int {
        guard let expiryDate = credentialsManager.expiryDate,
              let startDate = credentialsManager.startDate
        else {
            return 0
        }

        let difference = Calendar.current.dateComponents([.day], from: startDate, to: expiryDate)
        return difference.day ?? 0
    }
}
