import SwiftUI
import Constants
import ConnectionManager
import ExternalLinkManager
import UIComponents

final class SupportViewModel: ObservableObject {
    private let externalLinkManager: ExternalLinkManager
    private let faqLink = Constants.supportURL.rawValue
    private let newSupportRequest = Constants.newSupportRequest.rawValue
    private let githubIssueLink = Constants.ghIssuesLink.rawValue
    private let matrixLink = "https://nym.com/go/matrix"
    private let discordLink = Constants.discordLink.rawValue
    private let connectionManager: ConnectionManager

    let title = "settings.supportAndFeedback".localizedString

    @Binding var path: NavigationPath
    @Published var isResetVPNProfileDisplayed = false

    var sections: [SettingsListItemViewModel] {
        var newSections = [
            faqSectionViewModel(),
            emailSectionViewModel(),
            githubIssueViewModel(),
            matrixSectionViewModel(),
            discordSectionViewModel()
        ]
#if os(iOS)
        newSections.append(resetVPNProfileSectionViewModel())
#endif
        return newSections
    }

    init(
        path: Binding<NavigationPath>,
        connectionManager: ConnectionManager = ConnectionManager.shared,
        externalLinkManager: ExternalLinkManager = ExternalLinkManager.shared
    ) {
        _path = path
        self.connectionManager = connectionManager
        self.externalLinkManager = externalLinkManager
    }
}

// MARK: - Actions -
extension SupportViewModel {
    func resetVPNProfile() {
        connectionManager.resetVpnProfile()
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

    func  displayResetVPNProfileDialog() {
        isResetVPNProfileDisplayed = true
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
            action: { [weak self] in
                self?.openExternalURL(urlString: self?.faqLink)
            }
        )
    }

    func emailSectionViewModel() -> SettingsListItemViewModel {
        SettingsListItemViewModel(
            accessory: .arrow,
            title: "settings.getInTouch".localizedString,
            imageName: "email",
            position: SettingsListItemPosition(isFirst: true, isLast: true),
            action: { [weak self] in
                self?.openExternalURL(urlString: self?.newSupportRequest)
            }
        )
    }

    func githubIssueViewModel() -> SettingsListItemViewModel {
        SettingsListItemViewModel(
            accessory: .arrow,
            title: "feedback.githubIssue".localizedString,
            imageName: "github",
            position: SettingsListItemPosition(isFirst: true, isLast: true),
            action: { [weak self] in
                self?.openExternalURL(urlString: self?.githubIssueLink)
            }
        )
    }

    func matrixSectionViewModel() -> SettingsListItemViewModel {
        SettingsListItemViewModel(
            accessory: .arrow,
            title: "joinMatrix".localizedString,
            imageName: "matrix",
            position: SettingsListItemPosition(isFirst: true, isLast: true),
            action: { [weak self] in
                self?.openExternalURL(urlString: self?.matrixLink)
            }
        )
    }

    func discordSectionViewModel() -> SettingsListItemViewModel {
        SettingsListItemViewModel(
            accessory: .arrow,
            title: "joinDiscord".localizedString,
            imageName: "discord",
            position: SettingsListItemPosition(isFirst: true, isLast: true),
            action: { [weak self] in
                self?.openExternalURL(urlString: self?.discordLink)
            }
        )
    }

    func resetVPNProfileSectionViewModel() -> SettingsListItemViewModel {
        SettingsListItemViewModel(
            accessory: .empty,
            title: "settings.support.resetVpnProfile".localizedString,
            position: SettingsListItemPosition(isFirst: true, isLast: true),
            action: { [weak self] in
                self?.displayResetVPNProfileDialog()
            }
        )
    }
}
