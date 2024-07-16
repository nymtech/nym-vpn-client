import SwiftUI
import Constants
import ExternalLinkManager
import UIComponents

struct SupportViewModel {
    private let externalLinkManager: ExternalLinkManager
    private let faqLink = Constants.supportURL.rawValue
    private let emailLink = Constants.emailLink.rawValue
    private let matrixLink = "https://matrix.to/#/%23NymVPN:nymtech.chat"
    private let discordLink = Constants.discordLink.rawValue

    let title = "support".localizedString

    @Binding var path: NavigationPath
    var sections: [SettingsListItemViewModel] {
        [
            faqSectionViewModel(),
            emailSectionViewModel(),
            matrixSectionViewModel(),
            discordSectionViewModel()
        ]
    }

    init(path: Binding<NavigationPath>, externalLinkManager: ExternalLinkManager = ExternalLinkManager.shared) {
        _path = path
        self.externalLinkManager = externalLinkManager
    }
}

// MARK: - Navigation -
extension SupportViewModel {
    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }

    func openExternalURL(urlString: String?) {
        try? externalLinkManager.openExternalURL(urlString: urlString)
    }
}

// MARK: - Sections -

private extension SupportViewModel {
    func faqSectionViewModel() -> SettingsListItemViewModel {
        SettingsListItemViewModel(
            accessory: .arrow,
            title: "checkFAQ".localizedString,
            imageName: "faq",
            position: SettingsListItemPosition(isFirst: true, isLast: true),
            action: {
                openExternalURL(urlString: faqLink)
            }
        )
    }

    func emailSectionViewModel() -> SettingsListItemViewModel {
        SettingsListItemViewModel(
            accessory: .arrow,
            title: "sendEmail".localizedString,
            imageName: "email",
            position: SettingsListItemPosition(isFirst: true, isLast: true),
            action: {
                openExternalURL(urlString: emailLink)
            }
        )
    }

    func matrixSectionViewModel() -> SettingsListItemViewModel {
        SettingsListItemViewModel(
            accessory: .arrow,
            title: "joinMatrix".localizedString,
            imageName: "matrix",
            position: SettingsListItemPosition(isFirst: true, isLast: true),
            action: {
                openExternalURL(urlString: matrixLink)
            }
        )
    }

    func discordSectionViewModel() -> SettingsListItemViewModel {
        SettingsListItemViewModel(
            accessory: .arrow,
            title: "joinDiscord".localizedString,
            imageName: "discord",
            position: SettingsListItemPosition(isFirst: true, isLast: true),
            action: {
                openExternalURL(urlString: discordLink)
            }
        )
    }
}
