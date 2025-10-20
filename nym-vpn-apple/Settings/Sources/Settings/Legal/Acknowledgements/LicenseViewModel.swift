import SwiftUI
import AcknowList
import ExternalLinkManager

@MainActor public final class LicenseViewModel: ObservableObject {
    private let externalLinkManager: ExternalLinkManager

    @Binding private var path: NavigationPath

    let title = "legal.licence".localizedString

    @Published var acknowledgement: Acknow

    public init(
        path: Binding<NavigationPath>,
        details: LicenceDetails,
        externalLinkManager: ExternalLinkManager
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
    func fetchLicenseIfNecessary() {
        guard acknowledgement.text == nil,
              let repository = acknowledgement.repository,
              GitHubAPI.isGitHubRepository(repository)
        else { return }

        Task.detached(priority: .utility) { [weak self] in
            guard let self else { return }
            let result: Result<String, Error> = await withCheckedContinuation { cont in
                GitHubAPI.getLicense(for: repository) { res in
                    cont.resume(returning: res)
                }
            }
            switch result {
            case .success(let text):
                await MainActor.run {
                    self.acknowledgement = Acknow(
                        title: self.acknowledgement.title,
                        text: text,
                        license: self.acknowledgement.license,
                        repository: self.acknowledgement.repository
                    )
                }
            case .failure:
                await MainActor.run {
                    self.externalLinkManager.openExternalURL(repository)
                }
            }
        }
    }
}
