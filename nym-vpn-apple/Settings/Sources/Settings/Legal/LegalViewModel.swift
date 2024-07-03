import SwiftUI
import Constants
import ExternalLinkManager
import UIComponents

struct LegalViewModel {
    private let externalLinkManager: ExternalLinkManager
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

    init(path: Binding<NavigationPath>, externalLinkManager: ExternalLinkManager = ExternalLinkManager.shared) {
        self._path = path
        self.externalLinkManager = externalLinkManager
    }
}

// MARK: - Navigation -
extension LegalViewModel {
    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }

    func openExternalURL(urlString: String?) {
        // TODO: log error
        // TODO: fix opening settings after macos after macOS Sequoia release
        // https://www.reddit.com/r/SwiftUI/comments/16ibgy3/settingslink_on_macos_14_why_it_sucks_and_how_i/
        try? externalLinkManager.openExternalURL(urlString: urlString)
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
