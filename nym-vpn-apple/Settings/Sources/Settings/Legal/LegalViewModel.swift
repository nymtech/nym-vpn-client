import SwiftUI
import Constants
import UIComponents

struct LegalViewModel {
    private let termsOfUseLink = Constants.termsOfUseURL.rawValue
    private let privacyPolicyLink = Constants.privacyPolicyURL.rawValue
    #if os(iOS)
    private let licencesLink = UIApplication.openSettingsURLString
    #endif
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
        if !path.isEmpty { path.removeLast() }
    }

    func openExternalURL(urlString: String?) {
        #if os(iOS)
        guard let urlString, let url = URL(string: urlString) else { return }
        UIApplication.shared.open(url)
        #else
        NSApp.sendAction(Selector(("showSettingsWindow:")), to: nil, from: nil)
        #endif
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
                #if os(iOS)
                openExternalURL(urlString: licencesLink)
                #else
                openExternalURL(urlString: nil)
                #endif
            }
        )
    }
}
