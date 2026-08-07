import SwiftUI
import Constants
import ExternalLinkManager
import UIComponents

@MainActor final class SupportViewModel: ObservableObject {
    typealias SupportSection = SettingsSection<SupportSectionKind>

    private let externalLinkManager: ExternalLinkManager
    private let faqLink = Constants.supportURL.rawValue
    private let newSupportRequest = Constants.newSupportRequest.rawValue
    private let githubIssueLink = Constants.ghIssuesLink.rawValue
    private let telegramLink = Constants.telegramLink.rawValue
    private let matrixLink = Constants.matrixLink.rawValue
    private let discordLink = Constants.discordLink.rawValue
    let title = "settings.supportAndFeedback".localizedString

    @Binding var path: NavigationPath

    var sections: [SupportSection] {
        [
            SupportSection(kind: .faq, viewModels: faqRows()),
            SupportSection(kind: .contacts, viewModels: contactRows()),
            SupportSection(kind: .translations, viewModels: translateRows())
        ]
    }

    init(
        path: Binding<NavigationPath>,
        externalLinkManager: ExternalLinkManager
    ) {
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
    func faqRows() -> [SettingsListItemViewModel] {
        [
            SettingsListItemViewModel(
                accessory: .externalLink,
                title: "checkFAQ".localizedString,
                imageName: "faq",
                position: SettingsListItemPosition(isFirst: true, isLast: true),
                action: { [weak self] in
                    self?.openExternalURL(urlString: self?.faqLink)
                }
            ),
            SettingsListItemViewModel(
                accessory: .externalLink,
                title: "settings.getInTouch".localizedString,
                imageName: "email",
                position: SettingsListItemPosition(isFirst: true, isLast: true),
                action: { [weak self] in
                    self?.openExternalURL(urlString: self?.newSupportRequest)
                }
            )
        ]
    }

    func contactRows() -> [SettingsListItemViewModel] {
        [
            SettingsListItemViewModel(
                accessory: .externalLink,
                title: "feedback.githubIssue".localizedString,
                imageName: "github",
                position: SettingsListItemPosition(isFirst: true, isLast: true),
                action: { [weak self] in
                    self?.openExternalURL(urlString: self?.githubIssueLink)
                }
            ),
            SettingsListItemViewModel(
                accessory: .externalLink,
                title: "joinMatrix".localizedString,
                imageName: "matrix",
                position: SettingsListItemPosition(isFirst: true, isLast: true),
                action: { [weak self] in
                    self?.openExternalURL(urlString: self?.matrixLink)
                }
            ),
            SettingsListItemViewModel(
                accessory: .externalLink,
                title: "joinDiscord".localizedString,
                imageName: "discord",
                position: SettingsListItemPosition(isFirst: true, isLast: true),
                action: { [weak self] in
                    self?.openExternalURL(urlString: self?.discordLink)
                }
            ),
            SettingsListItemViewModel(
                accessory: .externalLink,
                title: "settings.chatOnTelegram".localizedString,
                imageName: "telegram",
                position: SettingsListItemPosition(isFirst: true, isLast: true),
                action: { [weak self] in
                    self?.openExternalURL(urlString: self?.telegramLink)
                }
            )
        ]
    }

    func translateRows() -> [SettingsListItemViewModel] {
        [
            SettingsListItemViewModel(
                accessory: .externalLink,
                title: "settings.helpTranslate.title".localizedString,
                subtitle: "settings.helpTranslate.subtitle".localizedString,
                systemImageName: "globe",
                action: { [weak self] in
                    self?.openExternalURL(urlString: Constants.crowdin.rawValue)
                }
            )
        ]
    }
}
