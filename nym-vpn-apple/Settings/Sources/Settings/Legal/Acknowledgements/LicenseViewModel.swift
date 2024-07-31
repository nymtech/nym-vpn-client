import SwiftUI
import AcknowList
import ExternalLinkManager

public final class LicenseViewModel: ObservableObject {
    private let externalLinkManager: ExternalLinkManager

    @Binding private var path: NavigationPath

    let title = "legal.licence".localizedString

    @Published var acknowledgement: Acknow

    public init(
        path: Binding<NavigationPath>,
        details: LicenceDetails,
        externalLinkManager: ExternalLinkManager = ExternalLinkManager.shared
    ) {
        _path = path
        self.acknowledgement = Acknow(
            title: details.title,
            text: details.text,
            license: details.license,
            repository: details.repository
        )
        self.externalLinkManager = externalLinkManager

        fetchLicenseIfNecessary()
    }

    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }
}

private extension LicenseViewModel {
    private func fetchLicenseIfNecessary() {
        guard acknowledgement.text == nil,
              let repository = acknowledgement.repository,
              GitHubAPI.isGitHubRepository(repository)
        else {
            return
        }

        GitHubAPI.getLicense(for: repository) { [weak self] result in
            guard let self else { return }
            switch result {
            case .success(let text):
                acknowledgement = Acknow(
                    title: acknowledgement.title,
                    text: text,
                    license: acknowledgement.license,
                    repository: acknowledgement.repository
                )
            case .failure:
                externalLinkManager.openExternalURL(repository)
            }
        }
    }
}
