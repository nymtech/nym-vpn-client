import SwiftUI
import Constants
import AppSettings
import UIComponents

struct FeedbackViewModel {
    private let githubIssueLink = "https://github.com/nymtech/nym-vpn-apple/issues"
    private let faqLink = Constants.supportURL.rawValue
    private let emailLink = "mailto:support@nymvpn.com"
    private let matrixLink = "https://matrix.to/#/%23NymVPN:nymtech.chat"
    private let discordLink = "https://discord.com/invite/nym"
    private let appSettings: AppSettings

    let title = "support".localizedString

    @Binding var path: NavigationPath
    var sections: [SettingsListItemViewModel] {
        [
            githubIssueViewModel(),
            feedbackSectionViewModel(),
            matrixSectionViewModel(),
            discordSectionViewModel()
        ]
    }

    init(path: Binding<NavigationPath>, appSettings: AppSettings = AppSettings.shared) {
        _path = path
        self.appSettings = appSettings
    }
}

// MARK: - Navigation -
extension FeedbackViewModel {
    func navigateBack() {
        path.removeLast()
    }

    func navigateToSurvey() {
        path.append(SettingsLink.survey)
    }

    func openExternalURL(urlString: String?) {
        guard let urlString, let url = URL(string: urlString) else { return }
        #if os(iOS)
        UIApplication.shared.open(url)
        #else
        NSWorkspace.shared.open(url)
        #endif
    }
}

// MARK: - Sections -

private extension FeedbackViewModel {
    func githubIssueViewModel() -> SettingsListItemViewModel {
        SettingsListItemViewModel(
            accessory: .arrow,
            title: "feedback.githubIssue".localizedString,
            imageName: "github",
            position: SettingsListItemPosition(isFirst: true, isLast: true),
            action: {
                openExternalURL(urlString: githubIssueLink)
            }
        )
    }

    func feedbackSectionViewModel() -> SettingsListItemViewModel {
        SettingsListItemViewModel(
            accessory: .arrow,
            title: "feedback.sendFeedback".localizedString,
            imageName: "sendEmail",
            position: SettingsListItemPosition(isFirst: true, isLast: true),
            action: {
                navigateToSurvey()
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
