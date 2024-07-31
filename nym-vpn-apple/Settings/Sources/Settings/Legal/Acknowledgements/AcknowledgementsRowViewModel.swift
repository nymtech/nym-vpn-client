import SwiftUI
import AcknowList
import ExternalLinkManager

public final class AcknowledgementsRowViewModel {
    private let externalLinkManager: ExternalLinkManager
    private var canFetchLicenseFromGitHub = true

    var acknowledgement: Acknow

    @Binding var path: NavigationPath

    var textExistsAndCanFetchLicence: Bool {
        acknowledgement.text != nil || canFetchLicenseFromGitHubAndIsGitHubRepository()
    }

    public init(
        acknowledgement: Acknow,
        path: Binding<NavigationPath>,
        externalLinkManager: ExternalLinkManager = ExternalLinkManager.shared
    ) {
        _path = path
        self.acknowledgement = acknowledgement
        self.externalLinkManager = externalLinkManager
    }

    func navigateToLicence() {
        path.append(
            SettingsLink.licence(
                details: LicenceDetails(
                    title: acknowledgement.title,
                    text: acknowledgement.text,
                    license: acknowledgement.license,
                    repository: acknowledgement.repository
                )
            )
        )
    }

    func openExternalBrowser() {
        guard let url = acknowledgement.repository else { return }
        externalLinkManager.openExternalURL(url)
    }

    func canOpenRepository() -> Bool {
        guard let repository = acknowledgement.repository,
              let scheme = repository.scheme
        else {
            return false
        }

        return scheme == "http" || scheme == "https"
    }
}

private extension AcknowledgementsRowViewModel {
    private func canFetchLicenseFromGitHubAndIsGitHubRepository() -> Bool {
        if canFetchLicenseFromGitHub,
           let repository = acknowledgement.repository {
            return GitHubAPI.isGitHubRepository(repository)
        } else {
            return false
        }
    }
}
