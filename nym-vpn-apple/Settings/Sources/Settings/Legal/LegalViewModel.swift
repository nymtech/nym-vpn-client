import SwiftUI
import UIComponents

struct LegalViewModel {
    private let termsOfUseLink = "https://nymvpn.com/en/terms"
    private let privacyPolicyLink = "https://nymvpn.com/en/privacy?type=apps"
    private let licencesLink = UIApplication.openSettingsURLString

    let title = "legal".localizedString

    @Binding var path: NavigationPath
    var viewModels: [SettingsListItemViewModel] {
        [
            termsOfUseViewModel(),
            privacyPolicyViewModel(),
            licencesViewModel()
        ]
    }
}

// MARK: - Navigation -
extension LegalViewModel {
    func navigateBack() {
        path.removeLast()
    }

    func openExternalURL(urlString: String?) {
        guard let urlString, let url = URL(string: urlString) else { return }
        UIApplication.shared.open(url)
    }
}

private extension LegalViewModel {
    func termsOfUseViewModel() -> SettingsListItemViewModel {
        SettingsListItemViewModel(
            accessory: .arrow,
            title: "legal.termsOfUse".localizedString,
            position: SettingsListItemPosition(isFirst: true, isLast: false),
            action: {
                openExternalURL(urlString: termsOfUseLink)
            }
        )
    }

    func privacyPolicyViewModel() -> SettingsListItemViewModel {
        SettingsListItemViewModel(
            accessory: .arrow,
            title: "legal.privacyPolicy".localizedString,
            position: SettingsListItemPosition(isFirst: false, isLast: false),
            action: {
                openExternalURL(urlString: privacyPolicyLink)
            }
        )
    }

    func licencesViewModel() -> SettingsListItemViewModel {
        SettingsListItemViewModel(
            accessory: .arrow,
            title: "legal.licences".localizedString,
            position: SettingsListItemPosition(isFirst: false, isLast: true),
            action: {
                openExternalURL(urlString: licencesLink)
            }
        )
    }
}
